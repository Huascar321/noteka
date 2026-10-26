# Proposal

## Why

El usuario quiere conectar su Google Drive para asociar recursos a entidades y abrir/previsualizar archivos. Se necesita OAuth de escritorio explícito, almacenamiento seguro y una primera lectura de metadatos sin otorgar acceso genérico al sistema.

## What Changes

- Login OAuth 2.0 de escritorio con PKCE, callback loopback y scope mínimo `drive.file`; tokens en Keychain.
- `noteka-auth google login|status|logout|files` para autorizar y listar hasta 20 archivos.
- `noteka-auth google pick|selected` con Google Picker API para que el usuario navegue y seleccione archivos de su Drive sin solicitar scopes restringidos. El Picker usa sólo `drive.file` y devuelve `picked_file_ids`; Noteka guarda la selección en una sesión separada de Keychain y consulta metadatos acotados.
- Asociaciones genéricas `external.link|unlink|list` en SQLite20: cualquier entidad puede vincular recursos por provider/cuenta/tipo/ID, con metadatos acotados, revisión y auditoría; desvincular no borra el archivo remoto.
- Preview read-only de Docs/Sheets por exportación (`text/plain`/`text/csv`), sin persistir contenido ni editar; la UI básica abre, previsualiza y desvincula.
- Sin Google Fotos. Sin escritura de celdas.

## Capabilities

### New Capabilities
- `google-drive`: autenticación Google y lectura de metadatos Drive.

### Modified Capabilities
- Ninguna.

## Impact

- Nueva crate/CLI `noteka-providers` (ya creada por 6a); SQLite/API/MCP no cambian.
- Requiere que el usuario cree un cliente OAuth de escritorio, habilite Drive API y Picker API, y complete el consentimiento.
- No se versionan client secrets ni tokens; no se envían datos sin autorización.
