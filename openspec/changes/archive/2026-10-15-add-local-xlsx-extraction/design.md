# Design

## Context

Baseline verificado: SQLite v10/API v1/51, UTF-8/PDF, MCP 23/43/44 y lector privado
XLSX archivado. `extraction_worker` deriva todo no UTF-8 a PDF; queue_integrity
interpreta no PDF como UTF-8; attachment_search_storage deriva el else a PDF.
Se deben separar esas tres fronteras. Baseline externo independiente: 355 archivos,
ocho binarios debug/release y catálogo; request_id se excluye al comparar datos.

## Goals / Non-Goals

**Goals:** extraer explícitamente un XLSX administrado, persistir su representación
sin pérdida y leerla por CLI/MCP; compartir cola/fence/budget; migrar/recuperar con
tests reales. Conservar contratos y validación/rendimiento FTS.

**Non-Goals:** FTS XLSX, Excel completo, evaluación de fórmulas, estilo visual,
inferencia de encabezados, UI/Home, pantalla nativa, modelos o red externa.

## Decisions

1. **Protocolo, no parser en core.** Core enlaza sólo noteka-xlsx-protocol y valida
   Reply/workbook antes de preparar persistencia. Los DTOs anidados adquieren
   JsonSchema con los atributos Serde existentes; no cambia wire v1 ni reglas.
   Supervisor XLSX sigue el patrón no bloqueante PDF, con presupuesto/fencing,
   stdin bytes sin rutas, env_clear/cwd temporal y Drop que mata/recolecta el hijo.
   Se mantiene PDF independiente para no ampliar este cambio a refactor genérico.
   El helper no es un sandbox FS/red OS ni una garantía de RSS.
2. **Representación canónica una vez.** `serde_json::to_string(Workbook)` v1,
   con orden de campos/colecciones del protocolo y <=1 MiB. Dividir UTF-8 en
   chunks consecutivos <=4096 bytes (<=512), sin cortar una celda pública.
   `extraction_xlsx_generations`: generation/job, versión, parser, SHA-256 de JSON,
   bytes, sheets/cells/chunks. `extraction_xlsx_chunks`: generation/job/ordinal/text.
   FKs compuestas con cascade, CHECKs y unique. No guardar copia JSON y filas de
   celdas a la vez. Alternativa rechazada: 4 KiB por celda (rechaza strings válidos)
   o reconstrucción numérica f64. Chunks son almacenamiento, no localizadores.
3. **Integridad antes de páginas.** Preflight de SQL cuenta bytes/filas y cruces
   antes de materializar; comprueba owner, ordinal, tamaño/chunk UTF-8, hash total,
   canonicalidad, DTO/schema/semántica y conteos. Leer original con hash administrado
    como los lectores previos. Open valida íntegramente los derivados XLSX nuevos
    en el snapshot efectivo antes de configurar escritura, sin helper; backup hace
    validación integral de extractores/índice según versión. No se amplían las
    fronteras históricas de open para UTF-8/PDF/FTS.
    Claim usa shape barato y snapshot previo completo fuera de lock. No reparación.
4. **Manifiesto público derivado, no segunda autoridad.** Se construye del workbook
   validado: versión/parser/hash/bytes/limitaciones/date_1904/styles, hojas con
   IDs/nombres/visibilidad/rangos/merges/cell_count. Items con ordinal global,
   locator kind xlsx_cell/hoja ID/A1 y celda tipada completa; orden hoja/fila/columna.
   Paginación 32 default, 1–64, cursor común con scope/op/job/generation/limit y
   ordinal de celda. Metadatos repetidos están acotados dentro de workbook <=1 MiB;
   response común <=2 MiB. Libro vacío devuelve manifiesto sin cursor. Ningún offset
   corresponde al ZIP. Fórmulas/cachés nunca se recalculan ni normalizan.
5. **Cola/retención compartidas.** xlsx_cells/config v1; MIME workbook OOXML u
   octet-stream y .xlsx case-insensitive. Import/open/enqueue no parsean. Dispatch
   exhaustivo por extractor soportado. Publicación atómica sustituye sólo su job;
   sum(bytes UTF8)+sum(bytes PDF)+sum(bytes JSON XLSX) <=32 MiB, sin doble contar
   manifiesto/celdas derivados del JSON. Fallos conservan generación anterior.
6. **v11 antes de persistir.** 011 sólo añade tablas y cambia ambas versiones.
   Init/migrate recorren 011; backup valida v10 sin consultar tablas v11. Rollback
   inyectado, v10 real y restore sin helper son aceptación, no sólo PRAGMA.
   No rollback in-place v11→v10: recuperar backup v10 a otra ruta con binario viejo.
   Versiones/configuraciones futuras se conservan sin ejecutar; no inventar un
   formato XLSX desconocido. Fixtures que escriben 11 como futuro pasan a version+1;
   fixtures downgrade eliminan sólo las tablas añadidas para recrear su versión.
   El registry SQL cerrado de lotes incluye 011 y schema_check exige las tablas
   v11; no ampliar la whitelist de mutaciones ni relajar comparación de DDL.
7. **Operación aditiva y permisos.** API v1/52; CLI y invoke mismos DTOs; MCP deriva
   schema/policy read. 24/44/45 tools, siete exclusiones conservadas. Sólo contrato
   nuevo y lista de extractores cambian en describe; whitelist batch no crece.
   Frontend recibe contratos generados, ninguna lógica visual nueva.
8. **FTS fuera de alcance, explícito.** `extraction.index` rechaza xlsx_cells antes
   de tocar el índice. Las ramas PDF/UTF8 son explícitas, nunca else formato ajeno.
   No ampliar enums públicos de búsqueda (preservar 51 esquemas). Repetir dos gates
   intactos <=300 ms y regresión completa; optimizar si falla, no aceptar excepción.

## Risks / Trade-offs

- El lector materializa hasta 1 MiB más DTOs para validar cada página; coste acotado,
  pero se mide con corpus y memoria, no se promete latencia universal.
- Duplicar patrón de supervisor exige pruebas independientes de pipes/hash/fence;
  evita arriesgar el runtime PDF validado en el mismo cambio de persistencia.
- JSON canónico queda versionado por manifest/protocolo; cambiar orden/campos exige
  migración o lector explícito, no reserializar datos desconocidos silenciosamente.
- Librerías/docs actuales: Schemars README/Serde compatibility consultados vía
  Context7; respeta tagged enums/deny_unknown_fields. No nueva librería de parsing.

## Migration Plan

Preparar ADR y 011 antes de tocar formato persistente. Verificar v10 creado por
binario baseline, generaciones/FTS/unknown blocks y fallos/backup. Implementar
DTO/storage/supervisor, conectar cola y exponer reader; actualizar fixtures y
contratos sin debilitar asserts. Corpus completo debug/release, CLI/MCP reales,
concurrencia/restore sin helper, regresión/gates/UI headless. Sync sólo tras cierre.

## Open Questions

Ninguna decisión de usuario pendiente. Distribución del sibling helper se valida
en desarrollo; firma/notarización y UI de extracción necesitan cambios separados.
