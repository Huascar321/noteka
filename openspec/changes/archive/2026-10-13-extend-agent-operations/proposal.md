# Proposal

## Why

El catálogo y el CLI ya ofrecen 38 operaciones, pero construir un sistema relacionado exige confirmar cada escritura por separado: el script de suscripción puede dejar colecciones parciales y no muestra un resultado evaluado antes de aplicarlo. Las fases 1–3 están cerradas; corresponde completar la frontera compartida para cambios complejos sin esperar la aceptación nativa de Home ni introducir un LLM.

## What Changes

- Añadir `changes.preview` y `changes.apply`, accesibles desde el núcleo y el CLI, con planes versionados, referencias explícitas a resultados anteriores, diff de dominio y comprobaciones finales sobre datos reales.
- Admitir inicialmente nueve mutaciones: `entity.create/update`, `collection.create`, `property.define`, `formula.set`, `record.create/attach/update` y `view.save`. Las comprobaciones finales reutilizan `database.query`; las demás operaciones siguen disponibles individualmente, no se anuncian como compatibles con lotes.
- Evaluar preview en una copia SQLite privada en memoria obtenida mediante la API de backup desde un snapshot de lectura. No escribir contenido/historial en el workspace, migrarlo ni importar/exportar archivos. Aplicar reejecutando las mismas reglas en una única transacción sobre la base original, nunca publicando esa copia.
- Exigir al aplicar el plan y las huellas de su preview: identidad/ruta, estado base, plan y efecto esperado. Rechazar cambios concurrentes, referencias inválidas y divergencias completas; mantener IDs de dominio entre preview y apply y no repetir automáticamente escrituras.
- Extender catálogo, contratos generados, ayuda/documentación y pruebas de paridad del CLI, sin romper solicitudes existentes. Publicar límites y exclusiones, incluidos workspaces que necesitan migración y esquemas SQL no reconocidos para lotes.
- Convertir la fixture sintética de suscripción en un recorrido CLI preview → verificación exacta → apply: dos colecciones, cinco miembros, seis asientos, BOB 145,69, cálculo vigente y vista Pendientes incrustada en una página original. Verificar rollback, concurrencia, reapertura y backup/restauración, sin programar una pantalla financiera especial.

## Non-goals

Harness/LLM, modelos, embeddings, extracción, MCP, permisos remotos, OAuth, acceso a datos personales, importación de Notion, nueva UI de aprobación, undo general, idempotencia persistente y una migración de almacenamiento. No agrupar importación de adjuntos, exportación, creación/restauración/backup/migración de workspace ni operaciones de filesystem en una supuesta transacción SQLite. Archivar/restaurar entidades, relaciones built-in y plantillas continúan como operaciones individuales; incorporarlas a lotes requerirá otro cambio. No cerrar Home ni reemplazar su aceptación nativa por pruebas headless.

## Capabilities

### New Capabilities

Ninguna: se amplía la frontera de operaciones compartidas existente.

### Modified Capabilities

- `core-operations`: catálogo/CLI completos para las operaciones implementadas, preview aislado, planes acotados con referencias explícitas, apply atómico y condicionado al preview, conservación y recorrido determinista de construcción.

## Impact

Contratos/dispatcher de `noteka-core`, helpers transaccionales de entidades/colecciones/fórmulas/vistas, apertura sin migración y nuevo módulo de lotes; mapeo `changes` en `noteka-cli`, generador de contratos y `scripts/subscription-fixture.mjs`. Se propone activar `hooks` en rusqlite existente para el presupuesto cooperativo, sin un nuevo motor o librería de persistencia. Tauri ya tiene un adaptador genérico hacia `execute_json`; no se añade una segunda implementación de dominio ni una ruta UI. Se mantienen SQLite **v7**, documentos **v1/v2/v3**, AST, importes exactos y backups existentes.

Decisiones del usuario: núcleo offline, Rust compartido y no ocupar su pantalla. Las fronteras del lote, referencias, huellas y copia en memoria son propuestas técnicas de este cambio; el diseño precisa su alcance. No hay dependencia externa de cuenta/proveedor. Las pruebas que requieren ventana nativa permanecen exclusivamente en Home y siguen pendientes.
