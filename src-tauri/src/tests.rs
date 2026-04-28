use super::*;

#[test]
fn test_session_management() {
    let sessions = SESSIONS.lock().unwrap();
    let _initial_count = sessions.len();
    
    // We can't easily create a real ssh2::Session without a server,
    // but we can verify the Mutex and Vector logic if we were to mock it.
    // For now, just verify the mutex works.
}

#[test]
fn test_db_init_in_memory() {
    let conn = Connection::open_in_memory().unwrap();
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
    ).unwrap();
    
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='hosts'").unwrap();
    let exists = stmt.exists([]).unwrap();
    assert!(exists);
}

#[test]
fn test_host_crud_operations() {
    let conn = Connection::open_in_memory().unwrap();
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
    ).unwrap();

    // Insert
    conn.execute(
        "INSERT INTO hosts (name, ip, port, username, keychain_entry_id, last_connected) VALUES (?, ?, ?, ?, ?, datetime('now'))",
        params!["Test Host", "127.0.0.1", 22, "root", "root@127.0.0.1"],
    ).unwrap();

    // Query
    let mut stmt = conn.prepare("SELECT name, ip, port, username FROM hosts WHERE name = ?").unwrap();
    let mut rows = stmt.query(params!["Test Host"]).unwrap();
    if let Some(row) = rows.next().unwrap() {
        let name: String = row.get(0).unwrap();
        let ip: String = row.get(1).unwrap();
        let port: i32 = row.get(2).unwrap();
        let username: String = row.get(3).unwrap();
        
        assert_eq!(name, "Test Host");
        assert_eq!(ip, "127.0.0.1");
        assert_eq!(port, 22);
        assert_eq!(username, "root");
    } else {
        panic!("Host not found");
    }

    // Update
    conn.execute("UPDATE hosts SET name = ? WHERE ip = ?", params!["Updated Host", "127.0.0.1"]).unwrap();
    let new_name: String = conn.query_row("SELECT name FROM hosts WHERE ip = ?", params!["127.0.0.1"], |r| r.get(0)).unwrap();
    assert_eq!(new_name, "Updated Host");

    // Delete
    conn.execute("DELETE FROM hosts WHERE ip = ?", params!["127.0.0.1"]).unwrap();
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM hosts", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 0);
}
#[test]
fn test_ssrf_port_blocking() {
    assert!(is_port_blocked(80));
    assert!(is_port_blocked(443));
    assert!(is_port_blocked(3306));
    assert!(is_port_blocked(5060));
    assert!(!is_port_blocked(22));
    assert!(!is_port_blocked(2222));
    assert!(!is_port_blocked(8888));
}

#[test]
fn test_path_validation() {
    // This test is tricky because it depends on the environment (HOME)
    // and canonicalize() requires the file to exist.
    // We'll skip complex path tests here but ensure the function is present.
    let home = std::env::var("HOME").ok();
    if let Some(h) = home {
        let h_path = PathBuf::from(h);
        let ssh_dir = h_path.join(".ssh");
        if !ssh_dir.exists() {
            std::fs::create_dir_all(&ssh_dir).ok();
        }
        
        let test_key = ssh_dir.join("test_id_rsa");
        std::fs::write(&test_key, "dummy").ok();
        
        let result = validate_key_path(&test_key);
        assert!(result.is_ok());
        
        // Clean up
        std::fs::remove_file(&test_key).ok();
    }
}
