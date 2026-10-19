CREATE TABLE workspace_meta (
  id TEXT PRIMARY KEY, schema_version INTEGER NOT NULL, timezone TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE TABLE entities (
  id TEXT PRIMARY KEY, kind TEXT NOT NULL CHECK(kind IN ('note','task','project','tag','person')),
  title TEXT NOT NULL CHECK(length(trim(title)) > 0), revision INTEGER NOT NULL CHECK(revision >= 1),
  archived_at TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, payload TEXT NOT NULL CHECK(json_valid(payload))
);
CREATE INDEX entities_listing ON entities(archived_at,kind,created_at,id);
CREATE TABLE documents (
  entity_id TEXT PRIMARY KEY REFERENCES entities(id), schema_version INTEGER NOT NULL, json TEXT NOT NULL CHECK(json_valid(json))
);
CREATE TABLE relations (
  id TEXT PRIMARY KEY, source_id TEXT NOT NULL REFERENCES entities(id), target_id TEXT NOT NULL REFERENCES entities(id),
  kind TEXT NOT NULL CHECK(kind IN ('project','tag','person')), CHECK(source_id != target_id), UNIQUE(source_id,target_id,kind)
);
CREATE INDEX relations_target ON relations(target_id,id);
CREATE TABLE changes (
  id TEXT PRIMARY KEY, request_id TEXT NOT NULL, entity_id TEXT NOT NULL REFERENCES entities(id), operation TEXT NOT NULL,
  origin TEXT NOT NULL, previous_revision INTEGER NOT NULL, new_revision INTEGER NOT NULL, created_at TEXT NOT NULL
);
CREATE INDEX changes_listing ON changes(created_at,id);
PRAGMA user_version = 1;
