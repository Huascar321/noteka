-- Estructura tabular Paddle derivada; nullable para generaciones no Paddle.
CREATE TABLE extraction_pdf_structures (
    generation_id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    version INTEGER NOT NULL CHECK(version=1),
    format TEXT NOT NULL CHECK(format='paddle_structure_v1'),
    source_sha256 TEXT NOT NULL CHECK(length(source_sha256)=64),
    structure_sha256 TEXT NOT NULL CHECK(length(structure_sha256)=64),
    structure_bytes INTEGER NOT NULL CHECK(structure_bytes BETWEEN 1 AND 1048576),
    page_count INTEGER NOT NULL CHECK(page_count BETWEEN 1 AND 20),
    table_count INTEGER NOT NULL CHECK(table_count BETWEEN 0 AND 512),
    structure_json TEXT NOT NULL CHECK(length(CAST(structure_json AS BLOB)) BETWEEN 2 AND 1048576),
    FOREIGN KEY(generation_id) REFERENCES extraction_generations(id) ON DELETE CASCADE
);
CREATE INDEX extraction_pdf_structures_job ON extraction_pdf_structures(job_id);
UPDATE workspace_meta SET schema_version=19;
PRAGMA user_version=19;
