# Integración PDFKit en curso

Cambio activo add-local-pdfkit-extraction. **No integrado todavía**: SQLite13,
API53, MCP25/45/46 y batch9 siguen vigentes. El runtime privado nuevo no se usa
desde las operaciones del producto y no abre workspace ni publica resultados.
Home9/17, buffers/interfaz y originales permanecen intactos.

Baseline durable542 archivos/22 builds, evidencia externa indicada por
~/.local/share/noteka-verification/active-pdfkit-extraction-path.txt.

## Entregas parciales verificables

- Fixtures reales13 vacía/mixta creadas con CLI anterior durable antes de DDL.
  Cuatro generaciones UTF8/PDF/XLSX/OCR, tres selecciones y backup/restore exactos
  en schema/filas de39 tablas/originales; catálogo53 conservado.
- Runtime Rust privado: bytes, header NKPDK001, SHA esperado, respuesta completa,
  errores tipados, guardia genérica preservada, deadline1..5000ms y pipes8MiB/8KiB.
  Hijo con red denegada, CWD temporal/entorno mínimo; sólo su Child se recolecta.
  Siete tests por perfil: cuatro verifican bloqueo/excesos/errores/respuesta hostil
  y cancelación/deadline/recolección, tres verifican el formato canónico. Los22
  PDF reales por perfil pasaron mediante consumidor Rust, preparación canónica y
  recarga íntegra. Clippy -D warnings y fmt aprobados.

Comandos de desarrollo, con toolchain local ya disponible y evidencia externa:

```sh
cargo test --offline -p noteka-pdfkit-runtime --all-targets
cargo test --offline --release -p noteka-pdfkit-runtime --all-targets
cargo build --offline -p noteka-pdfkit-runtime --example consume_pdfkit
cargo build --offline --release -p noteka-pdfkit-runtime --example consume_pdfkit
```

consume_pdfkit es una sonda de desarrollo: recibe ruta de helper como argumento y
bytes del PDF por stdin, no una operación pública. Retorna Document validado o
rechazo tipado del helper; los errores de proceso no reflejan fuente. La ruta del
helper nunca formará parte del payload público propuesto.

## Estado del cambio

Dos de ocho tareas verificadas. El codec canónico está implementado/probado como
parte parcial de3.1; no se escribió014 ni se integraron tablas, cuota, jobs o API.

## Pendientes

Persistencia/014 y cuota, migración/rollback/clientes/versiones/unknown data,
cola/claim/cancelación/publicación, extraction.pdfkit_pages, CLI/MCP/catálogos y
packaging/bindings, regresión/gates/revisión/sync/archivo. Todo sigue como tareas
pendientes mientras no tenga evidencia. FTS/UI del formato y OCR dePDF son otros
recortes. El diseño de recuperación está en ADR0022 propuesto y los artefactos.

Las lecturas Python iniciales mode=ro de fixtures WAL sin sidecars fallaron. Se
conservaron logs y se leyó sólo el estado cerrado/checkpointed, comprobando ausencia
de WAL no vacío antes de usar immutable=1. No se usó para ignorar WAL con datos.
Las snapshots lógicas de fuente/restaurado son exactas; no se atribuye una causa
interna demostrada al fallo de apertura de SQLite.

El vencimiento de5a sigue fijo a2026-10-18 02:16 UTC. Si permanece abierta se
detiene el trabajo con baseline/evidencia y tareas pendientes; no se archiva parcial.
