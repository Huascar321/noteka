# Verificación del harness de agentes (5c)

Estado: implementación y aceptación técnica local aprobadas el 21/10/2026
(America/La_Paz). No se activó un modelo real ni un proveedor remoto. Las pruebas
usan un servidor de chat loopback simulado; acreditan contrato, herramientas,
citas, límites, cancelación y trazas, no calidad de inferencia.

## Cambio y superficie

`add-agent-harness` agrega `agent.ask`, `agent.cancel` y `agent.trace`, la
migración aditiva 018 y el bucle acotado. SQLite pasa de 17 a 18; API v1 de 59 a
62 operaciones; MCP 32/53/55; batch9 no cambia. El proveedor es configurable por
solicitud y no modifica el dominio.

## Evidencia ejecutada

- Rust: **407 aprobadas/14 ignoradas** por perfil debug/release. El harness aporta
  10 pruebas: cita válida, cita inventada, falta de cita, insuficiencia explícita,
  herramienta prohibida, protocolo inválido, proveedor inalcanzable, cancelación
  entre procesos, segunda ejecución concurrente, preview/apply y migración 018.
- SDK MCP: **28 pruebas por perfil**; catálogo 62 operaciones, tools 32/53/55,
  acciones puras y permisos.
- Frontend: 94 pruebas Node y `svelte-check` 0/0.
- Headless: 5.120 checks sin solicitudes externas (misma corrida que 5b).
- Smoke CLI debug/release: `agent ask` con retrieval + final, estado `succeeded`,
  cita verificada, 3 pasos y traza consultable; `retrieval.search` degrada a FTS
  cuando se apaga el servidor de embeddings. Sin red externa.
- Migración/backup: workspace v17 → v18 conserva dominio; backup/restore conserva
  una ejecución insertada y no reactiva reservas. El workspace v16 real migrado a
  18 también conserva 1 entidad/6 jobs y restaura correctamente.
- Gates originales (corrida final): attachment query p95 máximo 134,020834 ms/
  lock22,925292 ms; XLSX mixto 189,807041/16,415833 ms; límites 300/150 intactos.
- `cargo fmt --check`, Clippy `-D warnings`, `git diff --check`, contratos,
  `svelte-check` y host no-bundle sin ventana aprobados.

## Protocolo y política

El proveedor recibe `/v1/chat/completions` loopback y responde contenido JSON;
el harness acepta `tool_call` o `final`, rechaza campos desconocidos y sólo
permite `retrieval.search`, `entity.get`, `database.query`, `changes.preview` y
`changes.apply` con `allow_apply` y recibo de preview de la misma ejecución. No
hay SQL, shell, archivos, workspace, exportación ni extracción.

Las citas se verifican contra hits de `retrieval.search` de la ejecución y el
`quote` debe aparecer en el fragmento tras normalizar whitespace. Sin citas se
exige `insufficient_evidence=true`; una cita inventada o un texto ausente falla.
La traza guarda resúmenes acotados (≤1 MiB), no el contexto completo ni
credenciales. La concurrencia es una reserva con vencimiento; la cancelación se
consulta antes de cada llamada y herramienta.

## Pendientes reales

- Ejecutar un modelo local real y evaluar calidad de respuestas/citas; no se hizo
  ni se declara.
- Evaluar proveedores cloud (fase 6) y OAuth/Drive; fuera de alcance.
- Un plan grande puede exceder el límite de herramienta/contexto y se rechaza,
  no se trunca. El snapshot de lotes incluye trazas e índice semántico.

## Incidencias

El fallo inicial de `test:headless-bridge` y `test:headless-reader` provino de
arneses que usaban `target/` y schema 16, no del producto. Se corrigieron para
usar `NOTEKA_CLI` y aceptar el schema actual; las suites pasaron después. Los
logs y artefactos se conservan fuera de Git.
