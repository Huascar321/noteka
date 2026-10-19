# Tasks

## 1. Baseline and controller

- [x] 1.1 Conservar y verificar baseline durable de archivos/binarios/catálogo v12 antes de editar implementación; documentar contratos observados y limitaciones en guía/verificación. Verificar manifiestos/hashes y Home intacto. Trazabilidad: seis requisitos de local-extraction.
- [x] 1.2 Implementar controlador de selección/lecturas/páginas explícitas con lifetime y cupo físico compartido; tests de secuencia, página reemplazada, error/incompatibilidad, éxito/rechazo tardío, cancelación en cola y remount. Documentar snapshots y ausencia de polling. Trazabilidad: selección, consulta y lifetime.
- [x] 1.3 Implementar acciones independientes con CAS observado y recibos, invalidación tras éxito/error y ningún retry/rebase; tests exactos de enqueue/run/control/cancel/retry/put, transporte incierto, generación/índice concurrentes y cero fragmentos. Documentar control global/recuperación/límites. Trazabilidad: acciones de cola y selección de generación.

## 2. Desktop integration

- [x] 2.1 Integrar panel separado de Search en nota con referencias únicas del buffer e ID manual, sin descargar originales ni cambiar Home/guardias. Preparar ambos helpers en hook local sin publicar; verificar extracción de referencias conocidas/desconocidas y check/build/host no-bundle; añadir guía de uso. Trazabilidad: selección y conservación de edición.
- [x] 2.2 Presentar estados/procedencia/configuración/índice/recibos/errores/límites reales escapados y controles por teclado en ambos temas. Verificar headless flujo UTF-8/PDF/XLSX y dedupe/pausa/retry/selección explícita con CLI real, y ausencia de acciones al montar/buscar. Trazabilidad: consulta, acciones, selección, presentación.
- [x] 2.3 Verificar headless concurrencia/conflictos, errores, navegación tardía/remount, paginación de trabajos, nota pendiente/guardias y viewport estrecho; snapshots prueban lecturas sin writes y originales/entidades conservados. Conservar logs/fallos fuera de Git. Trazabilidad: todos los escenarios del delta.

## 3. Integrated verification

- [x] 3.1 Ejecutar regresión fresca offline Rust debug/release, fmt/Clippy/contratos, frontend/controladores/puente/lotes/MCP y recorridos headless previos, con oracles/procesos XLSX/PDF pertinentes; registrar resultados reales y límites sin atribuir IPC nativo. Trazabilidad: conservación y aislamiento.
- [x] 3.2 Ejecutar dos gates mixtos y dos UTF-8/PDF intactos con validación/caps/corpus originales, query p95 <=300 ms y lock p95 <=150 ms; investigar fallos conservados sin relajar objetivos y registrar métricas. Trazabilidad: selección y regresión del núcleo.
- [x] 3.3 Revisar código/diseño y matriz de escenarios, diff contra baseline durable y hashes de Home/contratos/corpus/gates; actualizar plan/README/config/guías sólo con evidencia y validar OpenSpec estricto. Trazabilidad: seis requisitos completos.

## Workflow follow-up

- Sincronizar sólo el delta local-extraction implementado, contrastando todos los requisitos ajenos íntegros, y archivar este cambio cuando 9/9 tareas estén verificadas.
- Revalidar enlaces/list/status y conservar snapshot final fuera de Git. Home sigue activo 9/17 sin sync/archivo. Lectores y consulta agrupada se verifican en cambios separados; continúan ampliaciones/OCR según hoja de ruta. Este recorte no completa 5a.
