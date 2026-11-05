# Verificación: Inbox derivado de relaciones Project

Fecha: 2026-11-05. Alcance: `derive-inbox-from-projects` (núcleo Home/workflow,
UI de Tasks/Notes, docs). Sin cambios SQLite, contratos ni migraciones.

## Ejecutado

| Comando | Resultado |
| --- | --- |
| `cargo test --offline -p noteka-core --test home --test productivity` | 15 + 8 pruebas aprobadas. Cubren Inbox/Unclassified con relaciones Project, Tag/People sin efecto en Inbox, proyecto archivado, quitar vínculo, bandera inerte, done fuera y cursor incompatible. |
| `npm run check` | 0 errores, 0 warnings. |
| `npm run build` | Iconos, contratos y bundle OK; aviso de chunk >500 kB previo. |
| `npm run test:headless-inbox` | 12 checks: Task/Note Inbox explican la regla, Tasks sin tab Unclassified, sin control "In Inbox", vincular Project saca la fila de la vista, Tag no saca la nota y sigue excluida de Notes Unclassified, Notes conserva Unclassified en Home y página. Ejecutado dos veces (una con build previo y otra con el runner autocontenido que reconstruye `dist` y `noteka-cli`). |

App nativa (`npm run tauri dev`), verificada con la API de accesibilidad:

- Tasks → Inbox muestra el aviso "Pending tasks without a local Project relation.
  Linking one to a project removes it from this view." y el grupo "No project".
- La fila de vistas de Tasks ya no incluye Unclassified; Notes conserva la tab.
- Captura fuera de Git: `/tmp/noteka-inbox-native.png`.

## Fallos previos, no atribuidos a este cambio

- `cargo test --offline -p noteka-core` (suite lib completa): 264 aprobadas, 3
  rojas por hashes de contratos previos
  (`attachment_search_tests`, `pdf_contracts::v8_schemas...`,
  `xlsx_tests::all_51...`). `crates/noteka-core/src/contracts.rs` ya tenía 30
  líneas sin commit antes de este cambio y este cambio no lo modifica; el
  `describe()` cambió por ese trabajo previo.
- `npm run test:headless` (suite histórica) sigue roja por la condición previa
  documentada en
  [verification-auto-open-workspace.md](verification-auto-open-workspace.md):
  `tests/native/home.js` espera `.home-coverage`, que vive dentro de View
  options. El test nativo `smoke.js` se actualizó para la nueva regla de Inbox,
  pero no se ejecutó por esa condición.

## No ejecutado

- Suite nativa WKWebView completa y MCP end-to-end: este cambio no toca
  contratos ni el transporte MCP; el puente headless usa el binario CLI real
  reconstruido.
- Aceptación manual con datos de usuario más allá de la inspección de la ventana
  nativa descrita.
