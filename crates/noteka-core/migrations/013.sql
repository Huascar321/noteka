CREATE TABLE extraction_ocr_generations (
    generation_id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL UNIQUE,
    version INTEGER NOT NULL CHECK(version>0),
    profile TEXT NOT NULL,
    representation_sha256 TEXT NOT NULL CHECK(length(representation_sha256)=64),
    representation_bytes INTEGER NOT NULL CHECK(representation_bytes BETWEEN 1 AND 1048576),
    region_count INTEGER NOT NULL CHECK(region_count BETWEEN 0 AND 512),
    chunk_count INTEGER NOT NULL CHECK(chunk_count BETWEEN 1 AND 512),
    UNIQUE(job_id,generation_id),
    FOREIGN KEY(job_id,generation_id) REFERENCES extraction_generations(job_id,id) ON DELETE CASCADE
);
CREATE TABLE extraction_ocr_chunks (
    generation_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK(ordinal BETWEEN 0 AND 511),
    text TEXT NOT NULL CHECK(length(CAST(text AS BLOB)) BETWEEN 1 AND 4096),
    PRIMARY KEY(generation_id,ordinal),
    FOREIGN KEY(job_id,generation_id) REFERENCES extraction_ocr_generations(job_id,generation_id) ON DELETE CASCADE
);
UPDATE workspace_meta SET schema_version=13;
PRAGMA user_version=13;
