use noteka_core::{error::ErrorCode, workspace::Workspace};
use rusqlite::Connection;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;

#[test]
fn invalid_identity_and_missing_backup_schema_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("workspace");
    let w = Workspace::create(&path, None).unwrap();
    let backup = temp.path().join("backup");
    w.backup(&backup).unwrap();
    let conn = Connection::open(backup.join("noteka.sqlite")).unwrap();
    conn.execute_batch("DROP TABLE documents").unwrap();
    drop(conn);
    // Hash correcto no demuestra que sea un workspace con el schema declarado.
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(backup.join("manifest.json")).unwrap()).unwrap();
    manifest["sha256"] = Value::String(format!(
        "{:x}",
        Sha256::digest(fs::read(backup.join("noteka.sqlite")).unwrap())
    ));
    fs::write(backup.join("manifest.json"), manifest.to_string()).unwrap();
    let destination = temp.path().join("restored");
    assert!(matches!(
        Workspace::restore(&backup, &destination).unwrap_err().code,
        ErrorCode::InvalidBackup
    ));
    assert!(!destination.exists());
    let conn = Connection::open(path.join("noteka.sqlite")).unwrap();
    conn.execute("UPDATE workspace_meta SET id='invalid'", [])
        .unwrap();
    assert!(matches!(
        Workspace::open(&path).err().unwrap().code,
        ErrorCode::StorageError
    ));
}

#[test]
fn v7_backup_requires_template_table_even_with_valid_hash() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    let w = Workspace::create(&path, None).unwrap();
    let backup = temp.path().join("backup");
    w.backup(&backup).unwrap();
    let conn = Connection::open(backup.join("noteka.sqlite")).unwrap();
    conn.execute_batch("DROP TABLE templates;").unwrap();
    drop(conn);
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(backup.join("manifest.json")).unwrap()).unwrap();
    manifest["sha256"] = Value::String(format!(
        "{:x}",
        Sha256::digest(fs::read(backup.join("noteka.sqlite")).unwrap())
    ));
    fs::write(backup.join("manifest.json"), manifest.to_string()).unwrap();
    let destination = temp.path().join("restored");
    assert!(matches!(
        Workspace::restore(&backup, &destination).unwrap_err().code,
        ErrorCode::InvalidBackup
    ));
    assert!(!destination.exists());
}
