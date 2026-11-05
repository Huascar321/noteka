//! Regression coverage for the bundled SQLite engine and its WAL setup.
use rusqlite::{Connection, params};
use std::{sync::{Arc, Barrier}, time::Duration};

#[test]
fn bundled_engine_is_fixed_and_supports_existing_fts() {
    let db = Connection::open_in_memory().unwrap();
    let version: String = db.query_row("SELECT sqlite_version()", [], |row| row.get(0)).unwrap();
    assert_eq!(version, "3.54.1");
    db.execute_batch("CREATE VIRTUAL TABLE fts USING fts5(content); INSERT INTO fts VALUES ('local synthetic content');").unwrap();
    let matches: i64 = db.query_row("SELECT count(*) FROM fts WHERE fts MATCH 'synthetic'", [], |row| row.get(0)).unwrap();
    assert_eq!(matches, 1);
}

#[test]
fn concurrent_wal_writers_and_checkpoints_preserve_commits() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("synthetic.sqlite");
    let db = Connection::open(&path).unwrap();
    db.pragma_update(None, "journal_mode", "WAL").unwrap();
    db.pragma_update(None, "synchronous", "FULL").unwrap();
    db.execute_batch("CREATE TABLE counters (id INTEGER PRIMARY KEY, value INTEGER NOT NULL); INSERT INTO counters VALUES (1, 0);").unwrap();
    let barrier = Arc::new(Barrier::new(3));
    std::thread::scope(|scope| {
        for _ in 0..2 {
            let barrier = barrier.clone();
            let path = &path;
            scope.spawn(move || {
                let db = Connection::open(path).unwrap();
                db.busy_timeout(Duration::from_secs(5)).unwrap();
                db.pragma_update(None, "synchronous", "FULL").unwrap();
                barrier.wait();
                for _ in 0..50 {
                    db.execute("UPDATE counters SET value=value+1 WHERE id=?1", params![1]).unwrap();
                }
            });
        }
        let barrier = barrier.clone();
        let path = &path;
        scope.spawn(move || {
            let db = Connection::open(path).unwrap();
            db.busy_timeout(Duration::from_secs(5)).unwrap();
            barrier.wait();
            for _ in 0..50 {
                let _: (i64, i64, i64) = db.query_row("PRAGMA wal_checkpoint(PASSIVE)", [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).unwrap();
            }
        });
    });
    let count: i64 = db.query_row("SELECT value FROM counters WHERE id=1", [], |row| row.get(0)).unwrap();
    let integrity: String = db.query_row("PRAGMA integrity_check", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 100);
    assert_eq!(integrity, "ok");
}
