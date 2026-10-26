# Asociaciones de recursos externos

Noteka puede vincular cualquier entidad con recursos de proveedores externos sin
duplicar el archivo ni otorgar acceso genérico. La primera integración es Google
Drive mediante `noteka-auth google pick`, pero el modelo es genérico.

## Modelo

Migración020 agrega dos tablas:

- `external_resources`: identidad única por `provider`, `account_id`,
  `resource_type`, `resource_id`, metadata JSON y timestamps.
- `entity_external_resources`: asociación many-to-many entre una entidad y un
  recurso, con `created_at`.

Un mismo recurso puede estar asociado a varias entidades. Quitar una asociación
no borra el recurso de Drive ni la fila del recurso; sólo elimina el vínculo.

## Operaciones

### `external.link`

Input:

```json
{
  "entity_id": "<uuid>",
  "expected_revision": 3,
  "provider": "google_drive",
  "account_id": "<subject estable de Google>",
  "resource_type": "drive_file",
  "resource_id": "<ID de Drive>",
  "metadata": {
    "name": "Plan",
    "mimeType": "application/pdf",
    "modifiedTime": "2026-10-01T12:00:00Z",
    "webViewLink": "https://..."
  }
}
```

- Hace upsert del recurso y de la asociación.
- Incrementa la revisión de la entidad y registra `external.link` en `changes`
  cuando hay un cambio real.
- Es idempotente si la asociación y la metadata no cambian.
- `metadata` se limita a 64 KiB; `provider` 64 B, `account_id` 256 B,
  `resource_type` 64 B y `resource_id` 2048 B.

### `external.unlink`

```json
{
  "entity_id": "<uuid>",
  "expected_revision": 4,
  "id": "<id interno de external_resources>"
}
```

Quita sólo la asociación, incrementa la revisión y registra el cambio. No borra
el archivo remoto.

### `external.list`

```json
{"entity_id": "<uuid>", "limit": 50}
```

Devuelve `Page<ExternalResource>` con cursor. El límite predeterminado es 50 y el
máximo 200.

## Ejemplo con Google Picker

1. `noteka-auth google status` devuelve el `subject` estable de la cuenta.
2. `noteka-auth google pick` devuelve `picked_file_ids` y metadata de los
   archivos elegidos.
3. Se vincula el ID elegido a una entidad:

```sh
cat > /tmp/link.json <<'JSON'
{
  "entity_id": "<entity-id>",
  "expected_revision": 1,
  "provider": "google_drive",
  "account_id": "<subject>",
  "resource_type": "drive_file",
  "resource_id": "<picked-file-id>",
  "metadata": {"name": "Archivo", "mimeType": "application/pdf"}
}
JSON
noteka --workspace "$NOTEKA_WORKSPACE" external link --input /tmp/link.json
noteka --workspace "$NOTEKA_WORKSPACE" external list \
  --input <(printf '{"entity_id":"<entity-id>"}')
```

No se versionan IDs, nombres, metadata ni tokens de archivos reales.

## UI básica

La vista de una entidad muestra **Recursos externos** con:

- botón **Vincular desde Google Drive**, que invoca el Picker mediante un comando
  Tauri y vincula todos los archivos elegidos;
- **Abrir** para `webViewLink` permitidos de Google Docs/Drive/Sheets;
- **Desvincular**, que quita la asociación sin borrar el archivo remoto;
- **Previsualizar** para Google Docs/Sheets, que muestra texto plano o la primera
  hoja como CSV en un bloque de sólo lectura.

El comando Tauri `open_external` sólo acepta URLs `https://` de
`docs.google.com`, `drive.google.com` o `sheets.google.com`.

`noteka-auth google preview --id <id>` hace la exportación real. El contenido no
se guarda en SQLite ni en archivos; cada intento vuelve a consultar Drive y se
limpia antes de mostrar otro. El límite es 2 MiB. La exportación CSV no incluye
todas las hojas ni recalcula fórmulas.

## Límites actuales

- El preview es texto/CSV, no una réplica visual del documento; Sheets muestra
  sólo la primera hoja y no recalcula fórmulas.
- El camino Docs se verificó con un documento sintético; no se abrieron
  documentos personales.
- La metadata es informativa; no se refresca automáticamente.
- No se borra el recurso remoto al desvincular.
- El Picker desde la UI requiere que la app Tauri herede
  `NOTEKA_GOOGLE_CLIENT_ID`/`NOTEKA_GOOGLE_CLIENT_SECRET` del shell de desarrollo;
  no se leen credenciales de un archivo desde la GUI.
