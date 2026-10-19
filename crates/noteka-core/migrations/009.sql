CREATE TABLE extraction_pdf_generations (
    generation_id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL UNIQUE,
    version INTEGER NOT NULL CHECK(version>0),
    parser_name TEXT NOT NULL,
    parser_version TEXT NOT NULL,
    page_count INTEGER NOT NULL CHECK(page_count BETWEEN 1 AND 100),
    extracted_bytes INTEGER NOT NULL CHECK(extracted_bytes BETWEEN 0 AND 1048576),
    UNIQUE(job_id,generation_id),
    FOREIGN KEY(job_id,generation_id) REFERENCES extraction_generations(job_id,id) ON DELETE CASCADE
);
CREATE TABLE extraction_pdf_pages (
    generation_id TEXT NOT NULL,
    page_number INTEGER NOT NULL CHECK(page_number BETWEEN 1 AND 100),
    state TEXT NOT NULL CHECK(state IN ('text','no_text')),
    text_bytes INTEGER NOT NULL CHECK(text_bytes BETWEEN 0 AND 1048576),
    text_sha256 TEXT NOT NULL CHECK(length(text_sha256)=64),
    fragment_count INTEGER NOT NULL CHECK(fragment_count BETWEEN 0 AND 512),
    CHECK((state='text' AND text_bytes>0 AND fragment_count>0) OR (state='no_text' AND text_bytes=0 AND fragment_count=0)),
    PRIMARY KEY(generation_id,page_number),
    FOREIGN KEY(generation_id) REFERENCES extraction_pdf_generations(generation_id) ON DELETE CASCADE
);
CREATE TABLE extraction_pdf_fragments (
    generation_id TEXT NOT NULL,
    job_id TEXT NOT NULL,
    ordinal INTEGER NOT NULL CHECK(ordinal BETWEEN 0 AND 511),
    text TEXT NOT NULL CHECK(length(CAST(text AS BLOB)) BETWEEN 1 AND 4096),
    page_number INTEGER NOT NULL,
    text_byte_start INTEGER NOT NULL CHECK(text_byte_start>=0),
    text_byte_end INTEGER NOT NULL CHECK(text_byte_end>text_byte_start AND text_byte_end-text_byte_start=length(CAST(text AS BLOB))),
    PRIMARY KEY(generation_id,ordinal),
    FOREIGN KEY(job_id,generation_id) REFERENCES extraction_pdf_generations(job_id,generation_id) ON DELETE CASCADE,
    FOREIGN KEY(generation_id,page_number) REFERENCES extraction_pdf_pages(generation_id,page_number) ON DELETE CASCADE
);
UPDATE workspace_meta SET schema_version=9;
PRAGMA user_version=9;
