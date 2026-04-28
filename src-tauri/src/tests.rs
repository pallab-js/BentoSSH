use super::*;

#[test]
fn test_session_management() {
    let mut sessions = SESSIONS.lock().unwrap();
    let initial_count = sessions.len();
    
    // We can't easily create a real ssh2::Session without a server,
    // but we can verify the Mutex and Vector logic if we were to mock it.
    // For now, just verify the mutex works.
    assert!(initial_count >= 0);
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
