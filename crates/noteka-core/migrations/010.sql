CREATE TABLE extraction_search_control (
    id INTEGER PRIMARY KEY CHECK(id=1),
    version INTEGER NOT NULL CHECK(version>0),
    revision INTEGER NOT NULL CHECK(revision BETWEEN 1 AND 4294967295),
    profile TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
INSERT INTO extraction_search_control VALUES (1,1,1,'literal_unicode61_v1',strftime('%Y-%m-%dT%H:%M:%fZ','now'));
CREATE TABLE extraction_search_entries (
    slot INTEGER PRIMARY KEY CHECK(slot BETWEEN 1 AND 1024),
    job_id TEXT NOT NULL UNIQUE,
    generation_id TEXT NOT NULL UNIQUE,
    fragment_count INTEGER NOT NULL CHECK(fragment_count BETWEEN 0 AND 512),
    text_bytes INTEGER NOT NULL CHECK(text_bytes BETWEEN 0 AND 1048576),
    content_hash TEXT NOT NULL CHECK(length(content_hash)=64),
    indexed_at TEXT NOT NULL,
    FOREIGN KEY(job_id,generation_id) REFERENCES extraction_generations(job_id,id) ON DELETE CASCADE
);
-- BEGIN RESETTABLE
CREATE VIRTUAL TABLE extraction_search_fts USING fts5(job_id UNINDEXED,generation_id UNINDEXED,ordinal UNINDEXED,text,tokenize='unicode61',detail=full);
CREATE TRIGGER extraction_search_delete AFTER DELETE ON extraction_search_entries BEGIN
    DELETE FROM extraction_search_fts WHERE rowid BETWEEN (OLD.slot-1)*512+1 AND OLD.slot*512;
    UPDATE extraction_search_control SET revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=1;
END;
-- END RESETTABLE
UPDATE workspace_meta SET schema_version=10;
PRAGMA user_version=10;
