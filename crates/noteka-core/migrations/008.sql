CREATE TABLE extraction_control (
    id INTEGER PRIMARY KEY CHECK(id=1),
    version INTEGER NOT NULL CHECK(version>0),
    paused INTEGER NOT NULL CHECK(paused IN (0,1)),
    revision INTEGER NOT NULL CHECK(revision BETWEEN 1 AND 4294967295),
    updated_at TEXT NOT NULL
);
INSERT INTO extraction_control VALUES (1,1,1,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'));
CREATE TABLE extraction_jobs (
    id TEXT PRIMARY KEY,
    attachment_id TEXT NOT NULL REFERENCES attachments(id),
    source_sha256 TEXT NOT NULL,
    source_bytes INTEGER NOT NULL CHECK(source_bytes BETWEEN 0 AND 1048576),
    extractor TEXT NOT NULL,
    extractor_version INTEGER NOT NULL CHECK(extractor_version>0),
    config TEXT NOT NULL CHECK(json_valid(config) AND length(CAST(config AS BLOB))<=8192),
    config_hash TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('queued','running','succeeded','failed','cancelled','stale')),
    revision INTEGER NOT NULL CHECK(revision BETWEEN 1 AND 4294967295),
    attempts INTEGER NOT NULL CHECK(attempts BETWEEN 0 AND 4294967295),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    started_at TEXT,
    finished_at TEXT,
    error TEXT CHECK(error IS NULL OR (json_valid(error) AND length(CAST(error AS BLOB))<=4096)),
    generation_id TEXT REFERENCES extraction_generations(id) DEFERRABLE INITIALLY DEFERRED,
    UNIQUE(attachment_id,source_sha256,extractor,extractor_version,config_hash)
);
CREATE INDEX extraction_job_queue ON extraction_jobs(state,created_at,id);
CREATE INDEX extraction_job_source ON extraction_jobs(attachment_id,created_at,id);
CREATE TABLE extraction_runtime (
    id INTEGER PRIMARY KEY CHECK(id=1),
    job_id TEXT UNIQUE REFERENCES extraction_jobs(id),
    token TEXT,
    scope TEXT,
    control_revision INTEGER,
    lease_until TEXT,
    CHECK((job_id IS NULL AND token IS NULL AND scope IS NULL AND control_revision IS NULL AND lease_until IS NULL)
       OR (job_id IS NOT NULL AND token IS NOT NULL AND scope IS NOT NULL AND control_revision BETWEEN 1 AND 4294967295 AND lease_until IS NOT NULL))
);
INSERT INTO extraction_runtime(id) VALUES(1);
CREATE TABLE extraction_generations (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL UNIQUE REFERENCES extraction_jobs(id),
    attachment_id TEXT NOT NULL REFERENCES attachments(id),
    source_sha256 TEXT NOT NULL,
    extractor TEXT NOT NULL,
    extractor_version INTEGER NOT NULL CHECK(extractor_version>0),
    config TEXT NOT NULL CHECK(json_valid(config) AND length(CAST(config AS BLOB))<=8192),
    config_hash TEXT NOT NULL,
    source_bytes INTEGER NOT NULL CHECK(source_bytes BETWEEN 0 AND 1048576),
    covered_bytes INTEGER NOT NULL CHECK(covered_bytes=source_bytes),
    fragment_count INTEGER NOT NULL CHECK(fragment_count BETWEEN 0 AND 512),
    published_at TEXT NOT NULL,
    attempt INTEGER NOT NULL CHECK(attempt BETWEEN 1 AND 4294967295),
    UNIQUE(job_id,id)
);
CREATE TABLE extraction_fragments (
    generation_id TEXT NOT NULL,
    job_id TEXT NOT NULL REFERENCES extraction_jobs(id),
    ordinal INTEGER NOT NULL CHECK(ordinal BETWEEN 0 AND 511),
    text TEXT NOT NULL CHECK(length(CAST(text AS BLOB)) BETWEEN 1 AND 4096),
    byte_start INTEGER NOT NULL CHECK(byte_start>=0),
    byte_end INTEGER NOT NULL CHECK(byte_end>byte_start AND byte_end-byte_start=length(CAST(text AS BLOB))),
    line_start INTEGER NOT NULL CHECK(line_start>=1),
    line_end INTEGER NOT NULL CHECK(line_end>=line_start),
    PRIMARY KEY(generation_id,ordinal),
    FOREIGN KEY(job_id,generation_id) REFERENCES extraction_generations(job_id,id) ON DELETE CASCADE
);
UPDATE workspace_meta SET schema_version=8;
PRAGMA user_version=8;
