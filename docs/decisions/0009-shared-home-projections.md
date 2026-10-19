# ADR 0009 — Proyecciones Home compartidas y conservadoras

- Fecha: 12/10/2026.
- Estado: adoptada en `add-local-home-dashboard`.

## Contexto

El Home solicitado reúne cinco tipos existentes y dos zonas futuras (Goals/clima), sin duplicar identidad ni convertir built-ins en colecciones espejo. `entity.get` carga documentos y consultas por fila producirían N+1 y resúmenes mezclados. Workflows vigentes tienen otras semánticas.

## Decisión

`home.query`/`home query` comparte contratos y lectura Rust bajo snapshot SQLite Deferred. Proyecta referencias/metadatos conocidos, contexto y grupos paginados desde IDs originales; resúmenes completos por conjuntos, no por página. No expande documentos ni vistas incrustadas, escribe porcentajes o cambia SQLite v7/documentos v1–v3.

Today usa todo y due_date <= día civil del workspace; no modifica My Day (planned_date == día), Overdue (< día) o Active Projects (planned/doing/ongoing). Home Active es doing/ongoing. Topic se conserva. Reglas de las quince vistas y caps en [home.md](../home.md).

Progreso de proyecto = floor(100 * done / total), multiplicación comprobada; vacío null/Sin tareas, valor requerido futuro no disponible. Tag cuenta notas activas distintas y última por modificación/ID. Es cálculo propio, no fórmula Notion inferida. Campo futuro no relevante se conserva sin invalidar información conocida; nunca se usa parser de escritura estricto para convertir originales.

Cada sección controla una página y lifetime con cola de dos IPC. No promete snapshot de todo Home ni paginación congelada con escritores concurrentes. Guardias/editor/captura existentes ejecutan un solo commit. Tema sólo preferencia WebView, no dominio.

## Alternativas descartadas

- Colecciones espejo: nueva configuración e identidad implícita sin necesidad.
- `entity.list/get/relation.list` por fila: expansión de documentos, N+1 y ausencia de snapshot de resumen.
- Cambiar `workflow.query`: resultado incompatible y reinterpretación silenciosa de My Day/Active Projects.
- Persistir resúmenes: invalidación/migración y segunda autoridad.
- Mostrar pendientes como cifras/botones estáticos: apariencia sin comportamiento comprobable.

## Consecuencias y evidencia

Consulta acotada (10.000 candidatos/fuentes, 50.000 vínculos, 200 grupos); errores explícitos, no resumen recortado. Cursores versionados scoped a ruta/día/vista/límite, no tokens de seguridad. Sin migración, modelos o servicios externos. Home parcial mantiene la referencia solicitada; Goals/fotos/clima/Content/auxiliares se entregarán con contratos propios.

Tests: `crates/noteka-core/tests/home.rs`, snapshot y fecha determinista en `crates/noteka-core/src/home.rs`, paridad/proceso CLI concurrente en `crates/noteka-cli/tests/cli.rs`, controladores/tema en `tests/home-reads.mjs`, aserciones compartidas en `tests/native/home.js` más recorrido previo íntegro e intercalados de navegación/captura en `tests/headless-run.cjs`. El recorrido headless pasa sobre CLI/SQLite reales; aceptación nativa integrada pendiente. Medición/capturas y resultados frescos en [verification-home.md](../verification-home.md); [revisión final](../review-home.md).
