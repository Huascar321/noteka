CREATE VIRTUAL TABLE search_fts USING fts5(
  entity_id UNINDEXED, block_id UNINDEXED, revision UNINDEXED, title, text,
  tokenize='unicode61 remove_diacritics 2'
);
UPDATE workspace_meta SET schema_version=3;
PRAGMA user_version=3;
