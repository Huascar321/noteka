# Revisión de lectura de tablas PDF OCR

Revisión final aprobada8/8; sync/contraste/archivo a continuación.

| Requisitos del delta | Implementación y evidencia |
| --- | --- |
| Contrato aditivo de tabla y seis requisitos contractuales heredados | DTOs/dispatcher/CLI/scoped, comparación completa56 contratos y esquema, pruebas de catálogo y SDK28 por perfil; whitelist9 y errores idénticos. Las comparaciones históricas descuentan sólo adiciones nombradas. |
| Lectura explícita de tabla vigente | `pdf_ocr_table.rs`: transacción de lectura, original/canónico completos, biblioteca cerrada, manifiesto/página/motor/criterios/celdas y referencias exteriores; seis tests del núcleo y CLI real sin helpers. |
| Páginas ligadas a elección/generación | Hash de todos los criterios más cursor scoped, límites comunes, ordinales row-major/globales y cobertura diferenciada; cambio de criterio/generación/ruta rechazado y restore nuevo aceptado. |
| Tabla MCP y seis requisitos heredados de política/catálogo | Política compartida, READ ordenado, no helpers/permisos nuevos ni batch; modern/legacy/auto descubren/paginan y comparan resultados con CLI. |
| Lectura desktop explícita | Controlador/lifetime/cupo físico y panel existentes;101 frontend y286 headless, teclado, ambos temas, huecos/regiones múltiples/exteriores, texto hostil de transporte, errores y buffer. |

Los cuatro deltas contienen17 requisitos/43 escenarios: cinco requisitos nuevos
y doce modificados. La revisión de sincronización debe preservar escenarios
heredados y todos los bloques no afectados; no sustituir un main spec por un delta.

No hay persistencia nueva: SQLite y adjuntos originales mantienen autoridad.
La proyección no duplica FTS ni reejecuta OCR. Los originales/buffers y formatos
desconocidos sobreviven. Se conserva la biblioteca independiente y los helpers
Apple cerrados; Home9/17 permanece activo sin editar, sincronizar o archivar.

Aprobados:384 Rust por perfil,28 SDK por perfil,101 frontend,5146 headless,
contracts/fmt/Clippy y host no-bundle sin ventana. Dos pares consecutivos de gates
originales: query máximo182,232ms/lock18,445ms; lectura tabla21,365/19,241/22,166ms.
Diff/pins de Home/App/helpers y corpus/gates conservados; clientes56/57, unknowns
backup y pureza exactos. No se exceptuaron los fallos anteriores. Sólo después de
validar documentos sincronizar/contrastar estos cuatro deltas y archivar, auditar
toda5a y crear5b por dependencia.
