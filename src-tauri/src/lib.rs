#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lazy_static::lazy_static;
use rusqlite::{params, Connection};
use ssh2::Session;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;
use tauri::Emitter;
use thiserror::Error;
use uuid::Uuid;

/// Lock a mutex, recovering the guard even if the mutex was poisoned by a
/// panicking thread. For this app a poisoned lock is not corruption we need to
/// propagate as a crash — recovering keeps the UI responsive.
fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Error, Debug)]
pub enum SshError {
    #[error("Connection failed: {0}")]
    Connection(#[from] ssh2::Error),
    #[error("TCP connection failed: {0}")]
    Tcp(#[from] std::io::Error),
    #[error("Authentication failed for user {0}")]
    AuthFailed(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
}

impl From<SshError> for String {
    fn from(err: SshError) -> Self {
        match err {
            SshError::Connection(_) => "Connection failed. Please check host and port.".to_string(),
            SshError::Tcp(_) => "Network error. Please check your internet connection.".to_string(),
            SshError::AuthFailed(msg) => format!("Authentication failed: {}", msg),
            SshError::Database(_) => "Internal database error.".to_string(),
            SshError::Keyring(_) => "Security storage error.".to_string(),
        }
    }
}

#[derive(serde::Serialize)]
struct SavedHost {
    id: i32,
    name: String,
    ip: String,
    port: i32,
    username: String,
    keychain_entry_id: Option<String>,
    last_connected: Option<String>,
}

struct ActiveSession {
    session_id: String,
    _host: String,
    _username: String,
    session: Arc<Mutex<Session>>,
    channel: Arc<Mutex<ssh2::Channel>>,
    /// Signals the reader thread to stop and lets `ssh_disconnect` tear down cleanly.
    running: Arc<AtomicBool>,
}

lazy_static! {
    static ref SESSIONS: Arc<Mutex<Vec<ActiveSession>>> = Arc::new(Mutex::new(Vec::new()));
    static ref APP_HANDLE: Arc<Mutex<Option<tauri::AppHandle>>> = Arc::new(Mutex::new(None));
}

fn get_db_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let app_dir = std::path::Path::new(&home).join(".bentossh");
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).ok();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&app_dir, std::fs::Permissions::from_mode(0o700)).ok();
        }
    }
    let db_path = app_dir.join("bentossh.db");
    #[cfg(unix)]
    {
        if !db_path.exists() {
            // Create empty file to set permissions before SQLite opens it
            if let Ok(file) = std::fs::File::create(&db_path) {
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(std::fs::Permissions::from_mode(0o600))
                    .ok();
            }
        }
    }
    db_path
}

fn init_db() -> Result<Connection, SshError> {
    let conn = Connection::open(get_db_path())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS hosts (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            ip TEXT NOT NULL,
            port INTEGER DEFAULT 22,
            username TEXT NOT NULL,
            keychain_entry_id TEXT,
            last_connected TEXT
        )",
        [],
    )?;
    Ok(conn)
}

fn is_port_blocked(port: u16) -> bool {
    // SSRF Mitigation: Block sensitive internal ports and range of common non-SSH ports
    let blocked_ports = [
        21, 23, 25, 53, 80, 443, 3306, 5432, 6379, 8080, 8443, // Common services
        161, 162, // SNMP
        445, 137, 138, 139, // SMB/NetBIOS
        5060, 5061, // SIP
    ];
    blocked_ports.contains(&port)
}

fn validate_key_path(path: &Path) -> Result<PathBuf, SshError> {
    let canonical_path = path
        .canonicalize()
        .map_err(|_| SshError::AuthFailed("Invalid key path: file not found".into()))?;

    let home = std::env::var("HOME").map(PathBuf::from).ok();
    let ssh_dir = home.as_ref().map(|h| h.join(".ssh"));
    let app_dir = home.as_ref().map(|h| h.join(".bentossh"));

    let is_allowed = match (ssh_dir, app_dir) {
        (Some(s), Some(a)) => canonical_path.starts_with(s) || canonical_path.starts_with(a),
        (Some(s), None) => canonical_path.starts_with(s),
        (None, Some(a)) => canonical_path.starts_with(a),
        (None, None) => false,
    };

    if !is_allowed {
        return Err(SshError::AuthFailed(
            "Access denied: SSH keys must reside in ~/.ssh or ~/.bentossh".into(),
        ));
    }
    Ok(canonical_path)
}

#[tauri::command]
async fn ssh_connect(
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    key_path: Option<String>,
) -> Result<String, String> {
    if is_port_blocked(port) {
        return Err(format!(
            "Connection to port {} is restricted for security.",
            port
        ));
    }

    let host_clone = host.clone();
    let username_clone = username.clone();
    tokio::task::spawn_blocking(move || -> Result<String, SshError> {
        let tcp = TcpStream::connect(format!("{}:{}", host_clone, port))?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        if let Some(pass) = password {
            session.userauth_password(&username_clone, &pass)?;
        } else if let Some(key) = key_path {
            let path = std::path::PathBuf::from(key);
            let canonical_path = validate_key_path(&path)?;
            session.userauth_pubkey_file(&username_clone, None, &canonical_path, None)?;
        } else {
            return Err(SshError::AuthFailed("No credentials provided".into()));
        };

        let mut channel = session.channel_session()?;
        channel.request_pty("xterm-256color", None, None)?;
        channel.shell()?;

        // Non-blocking mode so the reader thread can poll without holding the
        // channel lock indefinitely, which would starve input and resize calls.
        session.set_blocking(false);

        let session_id = Uuid::new_v4().to_string();
        let session_arc = Arc::new(Mutex::new(session));
        let channel_arc = Arc::new(Mutex::new(channel));
        let running = Arc::new(AtomicBool::new(true));

        // Store session
        let mut sessions = lock_recover(&SESSIONS);
        sessions.push(ActiveSession {
            session_id: session_id.clone(),
            _host: host_clone,
            _username: username_clone,
            session: session_arc.clone(),
            channel: channel_arc.clone(),
            running: running.clone(),
        });
        drop(sessions);

        // Get app handle for emitting events
        let app_handle_clone = lock_recover(&APP_HANDLE).clone();

        // Spawn thread to read channel output and emit events to frontend
        let sid = session_id.clone();
        let reader = channel_arc.clone();
        let reader_running = running.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while reader_running.load(Ordering::Relaxed) {
                let read_result = {
                    let mut ch = lock_recover(&reader);
                    if ch.eof() {
                        break;
                    }
                    ch.read(&mut buf)
                    // channel lock released here, before sleeping
                };

                match read_result {
                    Ok(0) => break, // clean EOF
                    Ok(n) => {
                        let output = String::from_utf8_lossy(&buf[..n]).to_string();
                        if let Some(ref app_handle) = app_handle_clone {
                            let _ = app_handle.emit(&format!("ssh-output-{}", sid), output);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No data available yet; back off briefly to avoid a busy loop.
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
            reader_running.store(false, Ordering::Relaxed);
        });

        Ok(session_id)
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn ssh_send_input(session_id: String, data: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), SshError> {
        let sessions = lock_recover(&SESSIONS);
        for session in sessions.iter() {
            if session.session_id == session_id {
                let mut channel = lock_recover(&session.channel);
                // In non-blocking mode a write may report WouldBlock after a
                // partial write; loop until all bytes are flushed.
                let bytes = data.as_bytes();
                let mut written = 0;
                while written < bytes.len() {
                    match channel.write(&bytes[written..]) {
                        Ok(0) => break,
                        Ok(n) => written += n,
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        Err(e) => return Err(SshError::Tcp(e)),
                    }
                }
                let _ = channel.flush();
                return Ok(());
            }
        }
        Err(SshError::AuthFailed("Session not found".to_string()))
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn ssh_resize(session_id: String, cols: u32, rows: u32) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), SshError> {
        let sessions = lock_recover(&SESSIONS);
        for session in sessions.iter() {
            if session.session_id == session_id {
                let mut channel = lock_recover(&session.channel);
                channel.request_pty_size(cols, rows, None, None)?;
                return Ok(());
            }
        }
        Err(SshError::AuthFailed("Session not found".to_string()))
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn ssh_disconnect(session_id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        // Remove the session from the registry first.
        let removed = {
            let mut sessions = lock_recover(&SESSIONS);
            sessions
                .iter()
                .position(|s| s.session_id == session_id)
                .map(|pos| sessions.remove(pos))
        };

        // Signal the reader thread to stop, then close the channel gracefully.
        if let Some(active) = removed {
            active.running.store(false, Ordering::Relaxed);
            if let Ok(mut channel) = active.channel.lock() {
                let _ = channel.close();
                let _ = channel.wait_close();
            }
        }
    })
    .await
    .map_err(|e| e.to_string())
}

/// Run a one-shot command on a new exec channel and collect its output.
///
/// The interactive session runs in non-blocking mode, so we poll the exec
/// channel until it reports EOF instead of relying on a blocking read. This
/// avoids toggling the session-global blocking flag, which would interfere with
/// the shell reader thread sharing the same session.
fn exec_capture(session: &Session, command: &str) -> Result<String, SshError> {
    let mut channel = session.channel_session()?;
    channel.exec(command)?;

    let mut output = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match channel.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => output.extend_from_slice(&buf[..n]),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if channel.eof() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Err(SshError::Tcp(e)),
        }
    }
    let _ = channel.close();
    let _ = channel.wait_close();
    Ok(String::from_utf8_lossy(&output).to_string())
}

#[tauri::command]
async fn get_server_health(session_id: String) -> Result<(String, String), String> {
    tokio::task::spawn_blocking(move || -> Result<(String, String), SshError> {
        let sessions = lock_recover(&SESSIONS);
        let active = sessions
            .iter()
            .find(|s| s.session_id == session_id)
            .ok_or_else(|| SshError::AuthFailed("Session not found".into()))?;

        let session = lock_recover(&active.session);

        let cpu_output = exec_capture(
            &session,
            "top -bn1 | grep 'Cpu(s)' | awk '{print $2}' | cut -d'%' -f1",
        )?;
        let ram_output = exec_capture(
            &session,
            "free | grep Mem | awk '{printf \"%.1f\", ($3/$2)*100}'",
        )?;

        Ok((cpu_output.trim().to_string(), ram_output.trim().to_string()))
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn quick_action(session_id: String, action: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || -> Result<String, SshError> {
        let sessions = lock_recover(&SESSIONS);
        let active = sessions
            .iter()
            .find(|s| s.session_id == session_id)
            .ok_or_else(|| SshError::AuthFailed("Session not found".into()))?;

        let session = lock_recover(&active.session);

        let command = match action.as_str() {
            "restart_sshd" => "sudo systemctl restart sshd --no-pager -l",
            "tail_logs" => "tail -n 50 /var/log/syslog 2>/dev/null || tail -n 50 /var/log/messages 2>/dev/null || journalctl -n 50 --no-pager",
            "clear_cache" => "sync && echo 3 > /proc/sys/vm/drop_caches && echo 'Cache cleared'",
            _ => return Err(SshError::AuthFailed("Unknown action".to_string())),
        };

        exec_capture(&session, command)
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn save_host(
    name: String,
    ip: String,
    port: i32,
    username: String,
    password: Option<String>,
    key_path: Option<String>,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), SshError> {
        let conn = init_db()?;
        let entry_id = format!("{}@{}", username, ip);

        if let Some(pass) = password {
            let entry = keyring::Entry::new("bentossh:pass", &entry_id)?;
            entry.set_password(&pass)?;
        }

        if let Some(key) = key_path {
            let entry = keyring::Entry::new("bentossh:key", &entry_id)?;
            entry.set_password(&key)?;
        }

        conn.execute(
            "INSERT OR REPLACE INTO hosts (name, ip, port, username, keychain_entry_id, last_connected) VALUES (?, ?, ?, ?, ?, datetime('now'))",
            params![name, ip, port, username, entry_id],
        )?;
        Ok(())
    }).await.map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn get_hosts() -> Result<Vec<SavedHost>, String> {
    tokio::task::spawn_blocking(|| -> Result<Vec<SavedHost>, SshError> {
        let conn = init_db()?;
        let mut stmt = conn.prepare("SELECT id, name, ip, port, username, keychain_entry_id, last_connected FROM hosts ORDER BY last_connected DESC")?;
        let host_iter = stmt.query_map([], |row| {
            Ok(SavedHost {
                id: row.get(0)?,
                name: row.get(1)?,
                ip: row.get(2)?,
                port: row.get(3)?,
                username: row.get(4)?,
                keychain_entry_id: row.get(5)?,
                last_connected: row.get(6)?,
            })
        })?;
        let mut hosts = Vec::new();
        for host in host_iter {
            hosts.push(host?);
        }
        Ok(hosts)
    }).await.map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn delete_host(id: i32) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), SshError> {
        let conn = init_db()?;
        conn.execute("DELETE FROM hosts WHERE id = ?", params![id])?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn connect_saved_host(id: i32) -> Result<String, String> {
    let (ip, port, username, entry_id) = tokio::task::spawn_blocking(
        move || -> Result<(String, u16, String, Option<String>), SshError> {
            let conn = init_db()?;
            let mut stmt = conn
                .prepare("SELECT ip, port, username, keychain_entry_id FROM hosts WHERE id = ?")?;
            let mut rows = stmt.query(params![id])?;
            if let Some(row) = rows.next()? {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            } else {
                Err(SshError::AuthFailed("Host not found".to_string()))
            }
        },
    )
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e: SshError| String::from(e))?;

    let (password, key_path) = if let Some(ref eid) = entry_id {
        let p = keyring::Entry::new("bentossh:pass", eid)
            .and_then(|e| e.get_password())
            .ok();
        let k = keyring::Entry::new("bentossh:key", eid)
            .and_then(|e| e.get_password())
            .ok();
        (p, k)
    } else {
        (None, None)
    };

    ssh_connect(ip, port, username, password, key_path).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let mut handle = lock_recover(&APP_HANDLE);
            *handle = Some(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ssh_connect,
            ssh_send_input,
            ssh_resize,
            ssh_disconnect,
            get_server_health,
            quick_action,
            save_host,
            get_hosts,
            delete_host,
            connect_saved_host
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests;
