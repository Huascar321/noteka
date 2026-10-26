-- Recursos externos genéricos y asociaciones many-to-many con entidades.
CREATE TABLE IF NOT EXISTS external_resources (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL CHECK(length(CAST(provider AS BLOB)) BETWEEN 1 AND 64),
    account_id TEXT NOT NULL CHECK(length(CAST(account_id AS BLOB)) BETWEEN 1 AND 256),
    resource_type TEXT NOT NULL CHECK(length(CAST(resource_type AS BLOB)) BETWEEN 1 AND 64),
    resource_id TEXT NOT NULL CHECK(length(CAST(resource_id AS BLOB)) BETWEEN 1 AND 2048),
    metadata TEXT NOT NULL CHECK(length(CAST(metadata AS BLOB)) BETWEEN 2 AND 65536),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(provider, account_id, resource_type, resource_id)
);
CREATE TABLE IF NOT EXISTS entity_external_resources (
    entity_id TEXT NOT NULL REFERENCES entities(id),
    external_resource_id TEXT NOT NULL REFERENCES external_resources(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL,
    PRIMARY KEY(entity_id, external_resource_id)
);
CREATE INDEX IF NOT EXISTS entity_external_resources_entity
    ON entity_external_resources(entity_id, created_at);
UPDATE workspace_meta SET schema_version=20;
PRAGMA user_version=20;
