CREATE TABLE attachments (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, mime TEXT NOT NULL,
  bytes INTEGER NOT NULL CHECK(bytes >= 0), sha256 TEXT NOT NULL, created_at TEXT NOT NULL
);
UPDATE workspace_meta SET schema_version=2;
PRAGMA user_version=2;
