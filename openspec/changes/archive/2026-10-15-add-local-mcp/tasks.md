# Tasks

Estado: **22/22 tareas completadas/verificadas**, dos deltas sincronizados y cambio archivado el **15/10/2026**. Source final7: 235 tests Rust por perfil, 17 SDK por perfil, tres rondas raw por perfil, regresiones PDF/concurrencia/frontend/host sin lanzar y **606 checks headless** aprobados. Dos gates de adjuntos posteriores al recorrido exacto FTS5 pasan query p95 **134,96–204,81 ms** (<=300 ms), lock máximo p95 **19,80 ms**; sin cambiar corpus/caps ni retirar validación. Todos los fallos anteriores se conservan. Matriz de 15 requisitos/45 escenarios, contratos completos, preservación y revisión documentados en `docs/verification-mcp.md`; validación OpenSpec no acredita ejecución. Home conserva 9/17 y sus ocho brechas nativas; no se habilitan pantalla, modelos ni proveedores.

## 1. Baseline and dependencies

- [x] 1.1 Tomar snapshot/manifiesto externo de inicio, inventariar los 51 contratos completos y guardar comparación con el CLI independiente v10/51; registrar comandos/toolchain y estado de Home en docs/verification-mcp.md. Verificar que el baseline no cambió desde la planificación o documentar cambios legítimos antes de editar; no usar un diff Git vacío como evidencia. Trazabilidad: C1/C4, L11.
- [x] 1.2 Añadir crate noteka-mcp y dependencias exactas rmcp 3.5.1 con features mínimas/cliente TS 2.3.1 sólo de desarrollo; preservar default-members y dependencias ajenas. Verificar build/check offline tras instalación de tooling, árbol/features/licencias del binario, Node/MSRV y ausencia del cliente en el bundle frontend; registrar versiones resueltas y comandos en docs/development.md e investigación del cambio. Trazabilidad: L1/L7/L11.

## 2. Scoped core admission and policy

- [x] 2.1 Implementar clasificación exhaustiva core de 51 operaciones y acciones mixtas, permisos de lectura/escritura/run/excluidas y metadata in-process separada de describe; añadir tests de todos los modos, siete exclusiones, deny-by-default y denegación antes de efectos. Verificar counts 23/43/44 y rechazo de run sin escritura; documentar tabla efectiva sin afirmar transporte funcionando. Trazabilidad: C3, L3/L4.
- [x] 2.2 Añadir scope no serializable con ruta/identidad/política privadas y entrada JSON Operation estricta, con request_id generado y origen confiable; reutilizar dispatcher/métodos genéricos con opener actual main READ_ONLY para lecturas y RW sin CREATE/migrate para mutaciones. Verificar fixtures ausente/v9/v10/futura/inconsistente, overrides rechazados, campos/profundidad/bytes inválidos, permisos de apertura e identidad cambiada antes del uso real; tests CLI anteriores siguen pasando. Trazabilidad: C1/C2/C3, L2/L5/L6.
- [x] 2.3 Integrar expectativa scoped en admisión/conexiones/snapshots reales de preview/apply e índice/búsqueda sin reabrir por la ruta histórica; mantener fences, hash/recibos, TEMP, busy timeout, NO_MUTEX limitado y validación completa. Verificar cambio de identidad entre checks y apertura, CAS/recibo obsoleto, índice dañado sin reparación en lectura y put/reset autorizados con sus revisiones; conservar whitelist de lotes. Trazabilidad: C1/C2/C3, L2/L3.
- [x] 2.4 Cubrir todas las clases de lectura con main sólo legible, WAL confirmado, trabajos vencidos, formatos futuros, temporales y errores de integridad, comparando tablas/revisiones/originales antes/después; probar arranque por scope sin cambios físicos del workspace separadamente de SHM de consultas. Documentar límites de amenaza/API scoped en docs/mcp.md y ADR duradero; verificar ejemplos core y snapshot íntegro de contratos sin cambios. Trazabilidad: C1/C2/C4, L2/L3/L4.

## 3. Contract-derived discovery

- [x] 3.1 Implementar nombres/mapeo estable, inputSchemas del catálogo con refs locales y raíz object, filtrando control/index por acciones permitidas sin DTO paralelo. Verificar nombres únicos/ASCII/longitud, cobertura 51 y lista efectiva 23/43/44, schemas recursivos/resueltos y aceptación/rechazo reales por modo con validador independiente; actualizar guía de herramientas. Trazabilidad: L3/L4/L5, C4.
- [x] 3.2 Componer outputSchemas de Response v1 y dato específico conservando ramas de error, defs y nullabilidad sin colisiones. Validar todos los schemas y muestras reales de éxito/error, incluidos outputs genéricos y errores sin data; verificar que el catálogo core completo y contratos frontend/CLI siguen idénticos, y documentar wrapper frente al dato de dominio. Trazabilidad: L5/L6, C4.
- [x] 3.3 Implementar listado ordenado de ocho herramientas/página con cursor scoped <=2 KiB y único recurso estático noteka://catalog, sin datos/rutas personales, acceso URI genérico o suscripciones. Verificar todas las páginas/repetición/cursors ajenos/excesivos, counts completos, manifiesto por modo y discovery sin abrir/modificar workspace; documentar paginación y exclusiones en docs/mcp.md. Trazabilidad: L2/L4/L8.

## 4. Stdio runtime and lifecycle

- [x] 4.1 Implementar arranque --workspace obligatorio y flags inmutables, stdout sólo MCP y stderr mínimo/redactado; no sockets ni procesos gráficos. Verificar proceso real con red denegada, flags/rutas/versiones inválidos, códigos de arranque y sondeo sin efectos; añadir comandos reproducibles y configuración ficticia read-only a docs/mcp.md, sin instalarla en perfiles personales. Trazabilidad: L1/L2/L3/L11.
- [x] 4.2 Configurar lifecycle/modelos SDK para modern 2026-07-28 y legacy 2025-11-25, con capabilities mínimas y rechazo previo a efectos de versión/estado inválido. Añadir probes wire de discover sin initialize moderno, initialize/initialized legacy, metadata/resultType y errores de versión; verificar no anunciar revisiones o extensiones no probadas y documentar ambas eras. Trazabilidad: L7.
- [x] 4.3 Conectar tools/call a la entrada scoped, structuredContent/envelope/text equivalente, isError y errores MCP separados de dominio/permiso. Verificar not_found, DTO inválido, CAS, permission_denied, origen mcp, IDs de solicitud independientes y exceso de envelope/framing incluso tras commit real; no truncar ni reintentar y documentar error de entrega/reconciliación. Trazabilidad: L5/L6/L9, C3/C4.
- [x] 4.4 Acotar codec/líneas a 4 MiB, salida a 16 MiB, envelope a 2 MiB y canales/handlers/admisión conforme diseño, con un core activo/cero pendientes y server_busy antes de ejecución. Verificar líneas partidas/sin newline, exacto cap/+1, JSON malformado, saturación, cliente que no lee y mensajes escapados, sin cola/tareas ilimitadas ni bloquear EOF/cancelación; registrar límites reales en guía. Trazabilidad: L8/L9.
- [x] 4.5 Implementar cancelación del transporte antes de despacho y supresión de respuesta pendiente después, reteniendo el permiso dentro del trabajo síncrono hasta terminar. Verificar con barreras deterministas cancel-before, cancel-after, mutación confirmada/no confirmada, ausencia de respuesta tardía y segunda llamada rechazada mientras sigue activo, sin llamar extraction.cancel ni prometer rollback. Documentar semántica y carrera de respuesta ya enviada. Trazabilidad: L10/L9.
- [x] 4.6 Implementar cierre por EOF/error/backpressure terminal con admisión detenida, gracia <=30 s y limpieza de hijos propios, incluido control scoped de apagado del supervisor PDF si hace falta. Probar proceso/helper retenido, EOF durante lectura/mutación/run, broken pipe y recuperación de claim al reabrir; verificar no confiar en drop de un Future/process::exit para ejecutar destructores de otro thread y documentar límites ante terminación forzada. Trazabilidad: L10/L1, C1.

## 5. Independent client and integration

- [x] 5.1 Añadir harness del SDK cliente oficial local contra el binario real, scripts npm sin descargas en ejecución y fixtures sintéticas; recorrer pin modern, legacy explícito y auto con transcript raw y API SDK. Verificar discovery, schemas/páginas/resource, lectura/errores/denegaciones en los tres modos de permisos, versiones efectivas y ausencia de red/pantalla; documentar comando ejecutable y evidencia sin atribuir acceso de ChatGPT. Trazabilidad: L1–L9/L11, C4.
- [x] 5.2 Probar paridad CLI/MCP de dominio en la superficie común, incluidos entidades/documentos/relaciones, colecciones/registros/fórmulas/vistas/templates, consultas Home/workflows y suscripción exacta de BOB 145,69 por preview/apply. Verificar validación, total/IDs del lote, preview sin efectos, apply atómico e historial/origen, normalizando sólo metadatos documentados; incluir lote inválido/recibo obsoleto y conservación de futuros/originales. Registrar escenarios/resultados en docs/verification-mcp.md. Trazabilidad: C1/C2/C3/C4, L3/L5/L6/L11.
- [x] 5.3 Probar con cliente SDK enqueue/control/run/fragments UTF-8 y PDF real acotado, get/list, index get/put/reset y search con procedencia/cobertura/cursores; preparar/importar sólo por CLI sintético fuera de MCP. Verificar lectura sin recuperación, modo escritura sin ejecución, run sólo con ambos flags, helper ausente y datos previos conservados sin cambiar alcance/caps de extracción; registrar matriz y guía. Trazabilidad: L2/L3/L4/L6/L11, C2/C4.
- [x] 5.4 Ejecutar procesos CLI concurrentes con MCP: actualización CAS, escritor ocupado, cambio de dominio entre preview/apply, cancelación operativa durante run, cambio de identidad durante admisión y pérdida de respuesta tras commit. Verificar estados/errores/fences/revisiones reales, timeout de escritor 2 s, ausencia de duplicados/retries y ningún efecto en workspace ajeno; repetir rondas acotadas en debug/release y registrar todos los resultados. Trazabilidad: L2/L3/L9/L10, C1/C2/C3/C4.

## 6. Regression, measurements and review

- [x] 6.1 Ejecutar regresión fresca offline debug/release: fmt/check/Clippy, tests core/CLI/MCP y SDK, contratos/generación/examples, corpus/parser/CLI/contención PDF, migraciones/backup y concurrencia de adjuntos; frontend tests/build, host no-bundle y 606 checks headless con cero requests externos. Verificar/asserts previos sin retirar, Home/criterios nativos intactos y reportar warns/ignores/fallos con logs reales; no usar pantalla. Trazabilidad: C1/C4, L1/L11 y matriz completa.
- [x] 6.2 Medir release en M1 Max/64 GiB: arranque/discovery, RSS, latencia/overhead MCP frente a operaciones equivalentes CLI/core y saturación de framing/cancelación/cierre. Repetir benchmark existente de adjuntos por cambios de admisión, manteniendo target <=300 ms, caps y validación completa; registrar corpus, warmups/muestras, versiones y resultados fallidos, distinguiendo core de end-to-end sin promesa universal. Verificar métricas reproducibles y reportarlas en docs/verification-mcp.md. Trazabilidad: L9/L10/L11, C4.
- [x] 6.3 Revisar diff/manifiesto externo final y matriz individual de todos los requisitos/escenarios contra código, pruebas y diseño; verificar preservación de contratos/migraciones/originales/Home/archivos ajenos, links y documentación/ADR/roadmap/contexto coherentes con resultados reales. Ejecutar validación OpenSpec estricta del cambio y todas las specs, registrar brechas sin marcarlas completas y declarar listo para sincronizar/archivar sólo si las 22 tareas y criterios están verificados. Trazabilidad: ambas capabilities completas.

## Traceability

| ID local | Capability / Requirement |
| --- | --- |
| L1 | local-mcp / Servidor stdio local sin activación externa |
| L2 | local-mcp / Workspace fijo y admisión sin migración |
| L3 | local-mcp / Permisos locales efectivos por operación y acción |
| L4 | local-mcp / Superficie acotada y catálogo descubrible |
| L5 | local-mcp / Esquemas de herramientas derivados del contrato |
| L6 | local-mcp / Resultados estructurados y errores distinguibles |
| L7 | local-mcp / Ciclo de vida de ambas eras MCP |
| L8 | local-mcp / Descubrimiento paginado estable |
| L9 | local-mcp / Framing y admisión con recursos acotados |
| L10 | local-mcp / Cancelación y cierre sin garantías falsas de rollback |
| L11 | local-mcp / Interoperabilidad y consentimiento verificables |
| C1 | core-operations / Ejecución scoped sobre workspace actual |
| C2 | core-operations / Lecturas scoped sin mutaciones persistentes |
| C3 | core-operations / Política scoped validada antes de efectos |
| C4 | core-operations / Contrato compartido y versionado — modificación con cinco escenarios heredados íntegros |

## Workflow follow-up

- Sincronizar los dos deltas a main sólo después de verificar comportamiento, comparando requisitos/escenarios completos y conservando las doce capabilities previas y las cinco situaciones heredadas de C4. Archivar después y auditar archivo/links/manifiesto; no ejecutar estas acciones durante propuesta ni contarlas como implementación.
- Mantener Home activo 9/17, sin sincronizar ni archivar; aceptación nativa requiere nueva autorización específica. Administración/archivos por MCP, modelos/proveedores, HTTP/túneles y resto de 5a/5b/6c son cambios separados, no tareas escondidas de este checklist.
