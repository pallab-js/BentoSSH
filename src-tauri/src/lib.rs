#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ssh2::Session;
use std::net::TcpStream;
use std::path::PathBuf;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use thiserror::Error;
use rusqlite::{params, Connection};
use lazy_static::lazy_static;
use uuid::Uuid;
use tauri::Emitter;

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
        err.to_string()
    }
}

struct ActiveSession {
    session_id: String,
    host: String,
    username: String,
    session: Arc<Mutex<Session>>,
    channel: Arc<Mutex<ssh2::Channel>>,
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
            if let Ok(mut file) = std::fs::File::create(&db_path) {
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(std::fs::Permissions::from_mode(0o600)).ok();
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

#[tauri::command]
async fn ssh_connect(host: String, port: u16, username: String, password: Option<String>, key_path: Option<String>) -> Result<String, String> {
    // SSRF Mitigation: Block sensitive internal ports
    let blocked_ports = [21, 23, 25, 53, 80, 443, 3306, 5432, 6379, 8080, 8443];
    if blocked_ports.contains(&port) {
        return Err(format!("Connection to port {} is restricted for security.", port));
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
            let path = std::path::Path::new(&key);
            
            // Path Traversal Mitigation: Absolute path validation
            let canonical_path = path.canonicalize().map_err(|_| SshError::AuthFailed("Invalid key path: file not found".into()))?;
            let home = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
            let ssh_dir = home.join(".ssh");
            let app_dir = home.join(".bentossh");

            if !canonical_path.starts_with(&ssh_dir) && !canonical_path.starts_with(&app_dir) {
                 return Err(SshError::AuthFailed("Access denied: SSH keys must reside in ~/.ssh or ~/.bentossh".into()));
            }

            session.userauth_pubkey_file(&username_clone, None, &canonical_path, None)?;
        } else {
            return Err(SshError::AuthFailed(username_clone));
        };

        let mut channel = session.channel_session()?;
        channel.request_pty("xterm-256color", None, None)?;
        channel.shell()?;

        let session_id = Uuid::new_v4().to_string();
        let session_arc = Arc::new(Mutex::new(session));
        let channel_arc = Arc::new(Mutex::new(channel));

        // Store session
        let mut sessions = SESSIONS.lock().unwrap();
        sessions.push(ActiveSession {
            session_id: session_id.clone(),
            host: host_clone,
            username: username_clone,
            session: session_arc.clone(),
            channel: channel_arc.clone(),
        });

        // Get app handle for emitting events
        let app_handle_opt = APP_HANDLE.lock().unwrap();
        let app_handle_clone = app_handle_opt.clone();
        drop(app_handle_opt);

        // Spawn thread to read channel output and emit events to frontend
        let sid = session_id.clone();
        let reader = channel_arc.clone();
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                let mut ch = reader.lock().unwrap();
                match ch.read(&mut buf) {
                    Ok(n) if n > 0 => {
                        let output = String::from_utf8_lossy(&buf[..n]).to_string();
                        if let Some(ref app_handle) = app_handle_clone {
                            let _ = app_handle.emit_str(&format!("ssh-output-{}", sid), output);
                        }
                    }
                    Ok(_) => break,
                    Err(_) => break,
                }
            }
        });

        Ok(session_id)
    }).await.map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn ssh_send_input(session_id: String, data: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), SshError> {
        let sessions = SESSIONS.lock().unwrap();
        for session in sessions.iter() {
            if session.session_id == session_id {
                let mut channel = session.channel.lock().unwrap();
                channel.write_all(data.as_bytes())?;
                channel.flush()?;
                return Ok(());
            }
        }
        Err(SshError::AuthFailed("Session not found".to_string()))
    }).await.map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn ssh_resize(session_id: String, cols: u32, rows: u32) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), SshError> {
        let sessions = SESSIONS.lock().unwrap();
        for session in sessions.iter() {
            if session.session_id == session_id {
                let mut channel = session.channel.lock().unwrap();
                channel.request_pty_size(cols as u32, rows as u32, None, None)?;
                return Ok(());
            }
        }
        Err(SshError::AuthFailed("Session not found".to_string()))
    }).await.map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn ssh_disconnect(session_id: String) -> Result<(), String> {
    let mut sessions = SESSIONS.lock().unwrap();
    sessions.retain(|s| s.session_id != session_id);
    Ok(())
}

#[tauri::command]
async fn get_server_health(session_id: String) -> Result<(String, String), String> {
    tokio::task::spawn_blocking(move || -> Result<(String, String), SshError> {
        let sessions = SESSIONS.lock().unwrap();
        let active = sessions.iter().find(|s| s.session_id == session_id)
            .ok_or_else(|| SshError::AuthFailed("Session not found".into()))?;
        
        let session = active.session.lock().unwrap();
        
        let mut channel = session.channel_session()?;
        channel.exec("top -bn1 | grep 'Cpu(s)' | awk '{print $2}' | cut -d'%' -f1")?;
        let mut cpu_output = String::new();
        channel.read_to_string(&mut cpu_output)?;
        channel.close()?;
        channel.wait_close()?;

        let mut channel = session.channel_session()?;
        channel.exec("free | grep Mem | awk '{printf \"%.1f\", ($3/$2)*100}'")?;
        let mut ram_output = String::new();
        channel.read_to_string(&mut ram_output)?;
        channel.close()?;
        channel.wait_close()?;

        Ok((cpu_output.trim().to_string(), ram_output.trim().to_string()))
    }).await.map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn quick_action(session_id: String, action: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || -> Result<String, SshError> {
        let sessions = SESSIONS.lock().unwrap();
        let active = sessions.iter().find(|s| s.session_id == session_id)
            .ok_or_else(|| SshError::AuthFailed("Session not found".into()))?;
        
        let session = active.session.lock().unwrap();

        let command = match action.as_str() {
            "restart_sshd" => "sudo systemctl restart sshd --no-pager -l",
            "tail_logs" => "tail -n 50 /var/log/syslog 2>/dev/null || tail -n 50 /var/log/messages 2>/dev/null || journalctl -n 50 --no-pager",
            "clear_cache" => "sync && echo 3 > /proc/sys/vm/drop_caches && echo 'Cache cleared'",
            _ => return Err(SshError::AuthFailed("Unknown action".to_string())),
        };

        let mut channel = session.channel_session()?;
        channel.exec(command)?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.close()?;
        channel.wait_close()?;
        Ok(output)
    }).await.map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn save_host(name: String, ip: String, port: i32, username: String, password: Option<String>, key_path: Option<String>) -> Result<(), String> {
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
async fn get_hosts() -> Result<Vec<(i32, String, String, i32, String, Option<String>, Option<String>)>, String> {
    tokio::task::spawn_blocking(|| -> Result<Vec<(i32, String, String, i32, String, Option<String>, Option<String>)>, SshError> {
        let conn = init_db()?;
        let mut stmt = conn.prepare("SELECT id, name, ip, port, username, keychain_entry_id, last_connected FROM hosts ORDER BY last_connected DESC")?;
        let host_iter = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?))
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
    }).await.map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}

#[tauri::command]
async fn get_host_creds(id: i32) -> Result<(Option<String>, Option<String>), String> {
    tokio::task::spawn_blocking(move || -> Result<(Option<String>, Option<String>), SshError> {
        let conn = init_db()?;
        let mut stmt = conn.prepare("SELECT keychain_entry_id FROM hosts WHERE id = ?")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let entry_id: Option<String> = row.get(0)?;
            if let Some(ref eid) = entry_id {
                let pass = keyring::Entry::new("bentossh:pass", eid)?.get_password().ok();
                let key = keyring::Entry::new("bentossh:key", eid)?.get_password().ok();
                return Ok((pass, key));
            }
        }
        Ok((None, None))
    }).await.map_err(|e| e.to_string())
    .and_then(|r| r.map_err(|e| e.to_string()))
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let mut handle = APP_HANDLE.lock().unwrap();
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
            get_host_creds
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests;
