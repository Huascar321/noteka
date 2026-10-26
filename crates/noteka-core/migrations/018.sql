-- Trazas operativas del harness; no son fuente de verdad del dominio.
CREATE TABLE agent_runs (
    id TEXT PRIMARY KEY,
    version INTEGER NOT NULL CHECK(version=1),
    state TEXT NOT NULL CHECK(state IN ('running','succeeded','failed','cancelled','limited')),
    provider TEXT NOT NULL CHECK(length(CAST(provider AS BLOB)) BETWEEN 1 AND 64),
    model TEXT NOT NULL CHECK(length(CAST(model AS BLOB)) BETWEEN 1 AND 160),
    endpoint TEXT NOT NULL CHECK(length(CAST(endpoint AS BLOB)) BETWEEN 1 AND 2048),
    question TEXT NOT NULL CHECK(length(CAST(question AS BLOB)) BETWEEN 1 AND 8192),
    started_at TEXT NOT NULL,
    finished_at TEXT,
    lease_until TEXT NOT NULL,
    cancel_requested INTEGER NOT NULL DEFAULT 0 CHECK(cancel_requested IN (0,1)),
    step_count INTEGER NOT NULL DEFAULT 0 CHECK(step_count BETWEEN 0 AND 64),
    answer TEXT CHECK(answer IS NULL OR length(CAST(answer AS BLOB))<=65536),
    citations_json TEXT CHECK(citations_json IS NULL OR length(CAST(citations_json AS BLOB))<=262144),
    trace_json TEXT NOT NULL CHECK(length(CAST(trace_json AS BLOB)) BETWEEN 2 AND 1048576),
    error TEXT CHECK(error IS NULL OR length(CAST(error AS BLOB))<=4096)
);
CREATE INDEX agent_runs_started ON agent_runs(started_at,id);
CREATE INDEX agent_runs_state ON agent_runs(state,lease_until);
UPDATE workspace_meta SET schema_version=18;
PRAGMA user_version=18;
