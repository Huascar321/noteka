-- Las definiciones son originales: no se reescribe JSON de versiones anteriores.
CREATE TABLE templates (
  id TEXT PRIMARY KEY,
  collection_id TEXT NOT NULL REFERENCES collections(id),
  name TEXT NOT NULL,
  revision INTEGER NOT NULL CHECK (revision > 0),
  definition TEXT NOT NULL
);
CREATE INDEX templates_listing ON templates(collection_id,id);
UPDATE workspace_meta SET schema_version=7;
PRAGMA user_version=7;
