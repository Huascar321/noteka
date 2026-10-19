CREATE TABLE calculations (property_id TEXT PRIMARY KEY REFERENCES properties(id),expression TEXT NOT NULL CHECK(json_valid(expression)));
UPDATE workspace_meta SET schema_version=5;
PRAGMA user_version=5;
