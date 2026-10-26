# Verificación de asociaciones externas

Estado: **implementadas y verificadas localmente** el 22/10/2026. Cubren
persistencia, API/CLI, permisos MCP y migración; la UI y los previews siguen
pendientes.

## Modelo y migración

- Migración020: `external_resources` y `entity_external_resources`.
- Identidad única por `provider/account_id/resource_type/resource_id`.
- Relación many-to-many; metadata JSON acotada a 64 KiB.
- `external.link`, `external.unlink` y `external.list` agregadas al catálogo
  compartido de Rust.
- `link`/`unlink` incrementan la revisión de la entidad y registran el cambio;
  desvincular no borra el recurso ni modifica Drive.
- `v19 -> v20` se probó con tablas externas ausentes; la migración las crea y
  conserva filas originales.

## Pruebas

Suite de core:

- `link_is_idempotent_updates_metadata_and_shares_resource`: upsert, metadata,
  revisión, recurso compartido y desvinculación sin borrar la fila.
- `external_list_paginates_by_resource_id`: cursor y límite.
- `rejects_invalid_fields_metadata_and_missing_association`: provider vacío,
  metadata >64 KiB y asociación inexistente.
- `migrates_v19_to_external_tables`.

CLI real sobre workspace sintético:

1. `entity create` dejó una nota en revisión 1.
2. `external link` con `expected_revision: 1` devolvió la entidad en revisión 2
   y el recurso con metadata.
3. `external list` devolvió exactamente esa asociación.
4. `external unlink` con la revisión 2 devolvió la entidad en revisión 3.
5. `external list` volvió a devolver una página vacía.

No se usó una cuenta real de Drive en estas pruebas de asociación; son contratos
locales. El flujo Picker real se verificó por separado.

## Catálogo

- SQLite20.
- API v1/66 operaciones.
- MCP 34 herramientas de lectura / 57 con escritura / 59 con ejecución.
- batch9 sin cambios; las asociaciones no son mutaciones batchables todavía.

## Evidencia de regresión

- Workspace Rust: **433 aprobadas/16 ignoradas** por perfil debug; 0 fallos.
- `external_tests`: 4 pruebas.
- `noteka-providers`: 12 pruebas, incluidas las de preview y mapeo 401/403/404.
- `cargo check --workspace --all-targets` y Clippy del core/proveedores.
- `npm run contracts:generate` + `npm run contracts:check` + `npm run check`:
  esquema y tipos TypeScript actualizados, 0 errores.
- `npm run build` y `npm run test:headless`: 592 checks de producto + 14 de
  teclado, sin errores.
- `cargo test -p noteka-desktop`: 1 prueba del allowlist de URLs externas.
- Preview real de un Sheet seleccionado: CSV de 12.674 bytes/11 líneas, sin
  persistir contenido. 404 real, 403 simulado por loopback y conexión rechazada
  devolvieron `not_found`, `permission_denied` y `provider_http` respectivamente.
- El mapeo Docs -> `text/plain` y Sheets -> `text/csv` está cubierto por tests;
  además se previsualizó un Google Doc sintético real de 147 bytes (texto plano)
  seleccionado con Picker.
- OpenSpec estricto: 18/18 ítems válidos.

## Límites

- La UI básica permite vincular con Picker, previsualizar, abrir y desvincular;
  el Picker real desde la GUI no se ejecutó en esta sesión (el flujo de proveedor
  sí se verificó por CLI). La app debe heredar las variables OAuth del shell de
  desarrollo.
- El preview es texto/CSV, no una réplica visual; Sheets muestra sólo la primera
  hoja y no recalcula fórmulas. El camino Docs se verificó con contenido
  sintético, no con documentos personales.
- La metadata no se refresca sola; puede quedar obsoleta.
- El preview exporta contenido a memoria, no lo persiste; nunca escribe en el archivo remoto.
- No se versionan IDs, nombres, metadata ni tokens reales en este documento.
