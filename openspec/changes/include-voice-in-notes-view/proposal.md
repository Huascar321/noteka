# Proposal

## Why

La vista Notes inspeccionada en Notion admite explícitamente notas Voice Note; Noteka las excluye de Notes y las muestra sólo en Voice.

## What Changes

- Incluir notas Voice locales en la vista principal Notes sin quitar su acceso en Voice.
- Mantener `notes_library` y el resto de las consultas públicas previas. No inferir URL de documentos ni afirmar paridad completa con el filtro remoto.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: la vista Notes incluye Voice.
- `productivity-workflows`: proyección aditiva de notas generales.

## Impact

Rust, contratos generados y UI. Sin migración, cambios de datos, transcripción, sincronización ni archivo.
