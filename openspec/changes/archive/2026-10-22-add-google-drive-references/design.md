# Design

OAuth 2.0 Authorization Code + PKCE con `access_type=offline`, `prompt=consent` y scope `openid email profile https://www.googleapis.com/auth/drive.file`. Callback loopback `http://127.0.0.1:<puerto>/auth/callback`; state/verifier efímeros; token exchange form con client_id/secret opcional y code_verifier. El perfil se obtiene de userinfo; access/refresh se guardan en Keychain y se refrescan antes de expirar.

Configuración: `NOTEKA_GOOGLE_CLIENT_ID`, `NOTEKA_GOOGLE_CLIENT_SECRET` (si el cliente lo requiere), endpoints override sólo HTTPS o loopback de prueba. `google files` llama `drive/v3/files` con fields acotados; no descarga contenido ni crea asociaciones.

## Google Picker

`google pick` reutiliza el callback loopback y PKCE, pero abre una autorización separada con **sólo** `https://www.googleapis.com/auth/drive.file`, `prompt=consent` y `trigger_onepick=true`, como exige el flujo de escritorio de Google Picker. No incluye `openid`, `email` ni `profile` en esa petición. El callback devuelve `picked_file_ids`; se guarda una sesión Picker separada en Keychain con access/refresh token, scope y la lista de IDs, sin pisar la sesión de identidad.

`google selected` refresca esa sesión si expiró y consulta `files.get` para cada ID, con fields id/name/mimeType/modifiedTime/webViewLink. No enumera todo el Drive, no descarga contenido y no escribe. `google logout` borra ambas sesiones. Si no hay Picker API habilitada o el usuario cancela, se devuelve un error claro y no se conserva una selección parcial.

## Asociaciones externas

Migración020 agrega `external_resources` (identidad única por provider/account/resource) y `entity_external_resources` (many-to-many). `external.link` valida la revisión de la entidad, hace upsert de metadatos y de la asociación; `external.unlink` quita sólo la asociación; `external.list` pagina por ID de recurso. Crear, actualizar o quitar una asociación incrementa la revisión de la entidad y registra `external.link`/`external.unlink` en `changes`. No se borra el archivo remoto al desvincular. La metadata es JSON acotado a 64 KiB; provider/account/resource tienen límites explícitos.

La UI básica de entidad consume `external.list`, abre `webViewLink` mediante un comando Tauri con allowlist de hosts Google y desvincula. `google preview --id` exporta Docs como texto y Sheets como CSV con la sesión Picker, sin persistir contenido y con límite de 2 MiB; 401/403/404 y fallos de red se mapean a errores claros. La UI muestra el texto en un `<pre>` de sólo lectura y limpia la vista antes de cada intento. Pendiente: verificación real del camino Docs y archivo del cambio. No se implementa Google Fotos ni escritura.

La autenticación real requiere el cliente OAuth del usuario y su consentimiento; sin ellos sólo se prueban errores/configuración y almacenamiento en memoria.
