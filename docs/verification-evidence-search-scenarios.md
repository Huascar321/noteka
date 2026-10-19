# Matriz de Search con evidencia

Cambio cerrado [`add-local-evidence-search-ui`](../openspec/changes/archive/2026-10-16-add-local-evidence-search-ui/proposal.md), **17/17 tareas**.
Un delta, un requisito modificado y seis añadidos; diecinueve escenarios, incluido
el heredado de notas sin perder su texto. La matriz distingue código/tests de
resultados: consultar [verificación](verification-evidence-search-ui.md) para las
ejecuciones aprobadas. Regresión final y dos pares de gates aprobados; el primer
fallo de rendimiento está conservado y no se aceptó como excepción. La matriz
cubre siete requisitos y diecinueve escenarios, con 1.248 checks nuevos y los 606
heredados intactos; no sustituye la aceptación nativa pendiente de Home.

`N` = `tests/attachment-search.mjs`, cliente con promesas controladas.
`H` = `tests/headless-evidence.cjs`, navegador → CLI → SQLite reales.
`B` = `tests/headless-bridge.mjs`, aislamiento/fixtures/snapshots reales.
`F` = `scripts/prepare-evidence-search-fixture.py`, fuentes/golden independientes.
Los checks de H llevan nombre legible en `evidence-<tema>-results.json`; no son
pruebas de IPC/foco/cierre del host nativo. La regresión previa usa sus 606 checks
sin modificar los asserts en `tests/native/`.

## Traceability

| Requirement | Scenario | Implementación y verificaciones específicas |
| --- | --- | --- |
| Búsqueda de escritorio offline | Buscar y abrir sin red | `App.openSearch` conservado; H «Notas abre entidad/bloque originales» y regresión heredada `tests/native/smoke.js` 246–259. |
| Búsqueda de escritorio offline | Cambiar de ámbito sin cambiar el editor | `App.changeSearchScope`, panel separado; H «Alternar/buscar conserva buffer pendiente», filtros exclusivos, fuente de notas retenida al alternar. |
| Búsqueda de escritorio offline | Consultar adjuntos no es procesarlos | Panel sólo `extraction.search`; H registro de operaciones y estado inicial sin consulta; B fixture separada de lecturas. |
| Consulta de adjuntos desde escritorio con alcance explícito | Paridad de consulta y filtros | `AttachmentReader.search`; F 31 hits/45 llamadas; H `assertPage` para extractores, «Tres filtros AND», texto/IDs/orden/cobertura íntegros. |
| Consulta de adjuntos desde escritorio con alcance explícito | Consulta o filtro inválido | Validación Rust intacta; H multibyte >256 bytes, 17 términos, control no blanco y UUID inválido sin truncado; N filtros omitidos y errores. |
| Consulta de adjuntos desde escritorio con alcance explícito | Sin consulta y sin coincidencias son estados distintos | N estado inicial/cero hits; H «Estado inicial no es cero hits», «Cero hits con cobertura», error sin cobertura vigente. |
| Evidencia de adjuntos legible sin ejecución | Texto Unicode y procedencia completa | Panel texto `<pre>` y fuente completa; F reader/golden; H igualdad exacta y «Intento anterior visible». |
| Evidencia de adjuntos legible sin ejecución | Contenido activo es sólo texto | Interpolación Svelte sin HTML/linkificación; H «HTML/scripts/links inertes, sin interpretación», window sentinel y cero requests remotos. |
| Localizadores tipados de adjuntos en escritorio | Rangos UTF-8 del original | Panel `utf8_bytes`; F compara bytes/líneas con `extraction.fragments`; H `assertPage` UTF-8, multibyte y texto literal. |
| Localizadores tipados de adjuntos en escritorio | Página PDF sin geometría | Panel `pdf_page_text`; F reader PDF y golden con LF; H scope/limitaciones/página/rango exactos. |
| Localizadores tipados de adjuntos en escritorio | Campo XLSX parcial y caché | Panel `xlsx_cell_text`; F libro oculto y proyector independiente de seis campos; H «Campo XLSX», porción/caché/fórmula con ordinal/hoja/A1/bytes. |
| Cobertura de adjuntos visible en escritorio | Cobertura parcial sin recolección automática | Panel presenta cobertura del núcleo; F selección vacía/sin seleccionar/incompatible/sin generación `[4,1,1,1]`; H conteos y sin preparación UI. |
| Cobertura de adjuntos visible en escritorio | Selección vacía o fuentes fuera de página | Scope/validación explícitos y advertencias; H cero hits conserva cobertura, no total de coincidencias; F XLSX vacío seleccionado sin texto. |
| Paginación de adjuntos acotada y reinicio deliberado | Páginas sin acumulación | `AttachmentReader.next`, límite 20; N reemplazo/orden/cursor/reinicio; H 20+11 hits exactos, cursor final y reinicio manual. |
| Paginación de adjuntos acotada y reinicio deliberado | Cambio de índice o fuente inválida entre páginas | N conflicto/storage sin retry; B/H reset y fuente alterada deliberados; error retira página/cobertura, cada lectura conserva snapshot. |
| Paginación de adjuntos acotada y reinicio deliberado | Editar criterios invalida página anterior | `AttachmentReader.edit`; N y H retiran sin autoquery; H respuestas ya obtenidas retenidas mientras cambian texto/filtro. |
| Aislamiento de lecturas de evidencia en escritorio | Respuesta tardía de otro workspace o ámbito | Lifetime y pool compartido por montajes; N dispose/cola/rechazo tardío; H diez intercalados de éxito/error sobre texto/filtro/ámbito/sección/workspace. |
| Aislamiento de lecturas de evidencia en escritorio | Buffer y guardia preservados | Guardias centrales intactas; H consulta/inspección con nota sucia, cancelar/guardar/descartar, reapertura del buffer guardado. |
| Aislamiento de lecturas de evidencia en escritorio | Teclado y temas sin cambio de Home | Estilos scoped, controles nativos; H Enter consulta/paginación/procedencia, Space/Tab/foco, temas/capturas headless/wrap; hashes Home y regresión heredada. |

## Provenance and boundaries

Fuentes UI siempre sintéticas; no perfil personal ni contenido de Notion. Preparar
fixture, inyectar configuración futura o alterar/resetear una fuente para un caso
son acciones de **prueba**, no acciones que ejecute el panel. La instrumentación
verifica SQL lógico/esquema y hashes de archivos fuente/generaciones/índice; no
exige identidad física de WAL/SHM que SQLite puede coordinar al leer.

Los contratos y 283 fuentes congeladas se contrastan contra el baseline pre-UI v12.
No equivale a volver a comparar clientes o archivos v11 perdidos. El rendimiento
Rust se acepta por gates release intactos, no por tiempo del puente instrumentado.
Nada de esta matriz completa toda 5a/5b, modelos, remotos ni Home 9/17.
