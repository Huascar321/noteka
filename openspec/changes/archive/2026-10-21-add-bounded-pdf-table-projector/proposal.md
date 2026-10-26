# Proposal

## Why

La fase5a aún necesita tablas PDF. Las generaciones PDF OCR ya conservan regiones y geometría; hace falta un proyector acotado que pueda organizar esa evidencia sin inventar celdas o perder texto antes de integrarlo en el producto.

## What Changes

- Añadir una biblioteca Rust independiente para proyectar una página OCR ya validada en columnas elegidas y filas geométricas, con encabezado opcional explícito.
- Conservar evidencia original íntegra una vez y referenciar sus regiones por ordinal; cada región queda dentro de una celda o fuera del rectángulo elegido. Cruces de fronteras y filas ambiguas fallan completos.
- Verificar con golden sintético independiente, un PDF propio procesado por el OCR vigente y mediciones acotadas; incluir ejemplo JSON de desarrollo sin acceso a workspace o rutas.

## Non-goals

No operación pública, cola, persistencia, API57, CLI/MCP de producto, UI, FTS, detección genérica de tablas/columnas, celdas partidas o merges. No reconstruye lector PDFKit, cambia helpers nativos ni activa modelos. Integración y cierre completo de tablas PDF/5a requieren otro recorte;5b/5c esperan.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `local-extraction`: proyección independiente de regiones PDF OCR con columnas explícitas, conservación y ambigüedad acotadas.

## Impact

Nuevo crate independiente del workspace, dependiente sólo de serde/serde_json y protocolo OCRv2 existente. Núcleo/SQLite16/API56/MCP28/48/49/batch9, contratos, binarios y Home9/17 preservados. La biblioteca no verifica el archivo original ni concede vigencia del workspace: su integración futura debe revalidar canónico/fuente/ownership antes de llamar. Evidencia fuera de Git y límite20:11:26UTC del21/10.
