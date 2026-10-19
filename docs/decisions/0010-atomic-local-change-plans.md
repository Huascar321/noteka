# ADR 0010: planes locales de cambios con evaluación aislada y commit único

- Estado: adoptado; fase 4 implementada, verificada, sincronizada y archivada, 22/22 tareas.
- Fecha: 13 de octubre de 2026.
- Alcance: [`extend-agent-operations`](../../openspec/changes/archive/2026-10-13-extend-agent-operations/proposal.md), sin migración de SQLite v7 ni nuevo formato persistente.

## Contexto

Construir colecciones relacionadas con comandos individuales confirmaba cada mutación por separado. El caso sintético de suscripción ya calculaba centavos exactos, pero podía dejar estructura parcial y no incluía la página de vista original. El núcleo comparte reglas entre CLI y adaptador Tauri; Home tiene aceptación nativa pendiente que no es una dependencia de este cambio.

## Decisión

Nueve mutaciones y checks database.query finales mediante plan v1 cerrado. Sidecar de bindings tipados por slots y exports enumerados; sin interpolación de strings, SQL ni Operation recursivo. Helpers con conexión/root/MutationContext prestados; wrappers individuales siguen confirmando su Immediate. El ejecutor es el único dueño del commit del lote.

Preview copia un snapshot de lectura fijado mediante SQLite Backup a memoria, con page_size compatible e inclusión del WAL. La copia sólo evalúa y se descarta; apply reejecuta sobre el original bajo Immediate, comprueba identidad/esquema/base y compara efecto antes de confirmar. No migración oculta ni publicación de la copia. SHM puede cambiar por coordinación SQLite; no se confunde con mutación de dominio.

Identidades SHA-256 → UUID v8 por batch/step/rol/ordinal para conservar desempates exactos y referencias. Reloj de ejecución propio; normalización tipada de timestamps al comparar efecto, sin ocultar payload/civiles/cálculos/cobertura. Huella conservadora de schema y trece tablas originales crudas; contenido FTS derivado excluido. Rutas canónicas vinculan scope, incluido rechazo de un restore en otro destino.

Límites: 100 pasos, 10 checks, 512 bindings, 2 MiB input/output, snapshot 64 MiB, contención 2.000 ms y presupuesto cooperativo 5 s con hooks SQL y limpieza RAII. Originales/formatos desconocidos no se descartan; extensiones SQL no reconocidas fallan cerradas.

## Alternativas rechazadas

- CLI por paso o savepoints alrededor de wrappers que hacen commit: no produce atomicidad global.
- Rollback en original para preview: bloquea escritores y escribe WAL innecesariamente.
- Publicar la copia: pisa concurrencia y no garantiza adjuntos originales.
- UUIDs nuevos al aplicar: altera allocate y referencias.
- Read-set fino o sólo revisión de targets: no cubre dependencias, miembros nuevos ni lecturas negativas.
- Bindings por nombres/string aliases o segundo AST: duplica contratos o modifica texto arbitrario.

## Consecuencias y evidencia

Una escritura ajena puede invalidar un preview: es conservador, no rebase silencioso. Los recibos no autorizan ni firman; no son idempotencia persistente ni undo. No se guarda plan/recibo ni se inventa historial persistente de esquema. Checks certifican su filtro/filas, no todos los cálculos del workspace.

Pruebas SQLite reales cubren ownership, rollback, IDs, WAL/page_size, versiones/extensiones, huellas, deadlines, lectores independientes, error de almacenamiento antes de commit y formatos futuros. Catálogo/enum/CLI coinciden en 40 operaciones. Recorrido de 16 pasos mediante binario CLI real con red denegada: 14.569 centavos, mismos IDs, página referenciada, recálculo/pago y backup/restauración. [Verificación](../verification-phase-4.md): 115 tests Rust, cuatro CLI de suscripción y 606 checks headless; contratos/comandos/receta en [batches](../batches.md). No certifica IPC/cierre/foco/CSP nativos de Home ni inferencia local.
