-- Barrera de compatibilidad: el DDL y el perfil FTS v1 no cambian.
UPDATE workspace_meta SET schema_version=12;
PRAGMA user_version=12;
