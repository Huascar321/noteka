-- Recuperación semántica aditiva; originales y formatos anteriores se conservan.
CREATE TABLE semantic_profiles (
    fingerprint TEXT PRIMARY KEY CHECK(length(fingerprint)=64),
    version INTEGER NOT NULL CHECK(version=1),
    profile_json TEXT NOT NULL CHECK(length(CAST(profile_json AS BLOB)) BETWEEN 1 AND 8192),
    dimension INTEGER NOT NULL CHECK(dimension BETWEEN 1 AND 1024),
    created_at TEXT NOT NULL
);
CREATE TABLE semantic_generations (
    generation_id TEXT PRIMARY KEY,
    version INTEGER NOT NULL CHECK(version=1),
    fingerprint TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('building','active','retired','failed')),
    endpoint TEXT NOT NULL CHECK(length(CAST(endpoint AS BLOB)) BETWEEN 1 AND 2048),
    source_digest TEXT NOT NULL CHECK(length(source_digest)=64),
    source_count INTEGER NOT NULL CHECK(source_count BETWEEN 0 AND 1024),
    chunk_count INTEGER NOT NULL CHECK(chunk_count BETWEEN 0 AND 8192),
    text_bytes INTEGER NOT NULL CHECK(text_bytes BETWEEN 0 AND 8388608),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    activated_at TEXT,
    retired_at TEXT,
    error TEXT CHECK(error IS NULL OR length(CAST(error AS BLOB))<=4096),
    FOREIGN KEY(fingerprint) REFERENCES semantic_profiles(fingerprint) ON DELETE RESTRICT
);
CREATE INDEX semantic_generations_state ON semantic_generations(state,created_at);
CREATE UNIQUE INDEX semantic_single_building ON semantic_generations(state) WHERE state='building';
CREATE TABLE semantic_chunks (
    generation_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK(ordinal BETWEEN 0 AND 8191),
    job_id TEXT NOT NULL,
    attachment_id TEXT NOT NULL,
    source_generation_id TEXT NOT NULL,
    source_content_hash TEXT NOT NULL CHECK(length(source_content_hash)=64),
    extractor TEXT NOT NULL,
    extractor_version INTEGER NOT NULL CHECK(extractor_version>0),
    fragment_ordinal INTEGER NOT NULL CHECK(fragment_ordinal BETWEEN 0 AND 511),
    text TEXT NOT NULL CHECK(length(CAST(text AS BLOB)) BETWEEN 1 AND 4096),
    text_sha256 TEXT NOT NULL CHECK(length(text_sha256)=64),
    locator TEXT NOT NULL CHECK(length(CAST(locator AS BLOB)) BETWEEN 2 AND 8192),
    vector BLOB NOT NULL CHECK(length(vector) BETWEEN 4 AND 4096),
    PRIMARY KEY(generation_id,ordinal),
    FOREIGN KEY(generation_id) REFERENCES semantic_generations(generation_id) ON DELETE CASCADE
);
CREATE INDEX semantic_chunks_source ON semantic_chunks(generation_id,job_id);
CREATE TABLE semantic_control (
    id INTEGER PRIMARY KEY CHECK(id=1),
    version INTEGER NOT NULL CHECK(version=1),
    revision INTEGER NOT NULL CHECK(revision BETWEEN 1 AND 4294967295),
    active_generation_id TEXT,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(active_generation_id) REFERENCES semantic_generations(generation_id) ON DELETE RESTRICT
);
INSERT INTO semantic_control VALUES (1,1,1,NULL,strftime('%Y-%m-%dT%H:%M:%fZ','now'));
UPDATE workspace_meta SET schema_version=17;
PRAGMA user_version=17;
