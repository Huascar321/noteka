-- Barrera de compatibilidad: las tablas genéricas no autorizan a clientes14
-- a interpretar la proyección PDFKit nueva. No modifica ni reindexa datos.
UPDATE workspace_meta SET schema_version=15;
PRAGMA user_version=15;
