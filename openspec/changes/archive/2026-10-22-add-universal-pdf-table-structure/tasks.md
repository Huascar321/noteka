# Tasks

## 1. Worker y estructura

- [x] 1.1 Parsear tablas Paddle a estructura tipada con spans/texto y límites.
- [x] 1.2 Extender el supervisor para devolver páginas + estructura y validar hash/tamaño.
- [x] 1.3 Probar tabla real, HTML inválido, límites y ausencia.

## 2. Migración y almacenamiento

- [x] 2.1 Escribir migración 019 aditiva y actualizar creación/migración/schema/integridad/backup.
- [x] 2.2 Persistir estructura en `paddle_ocr` y leer/validar por job.
- [x] 2.3 Probar migración 18→19, backup/restore, generación previa y corrupción.

## 3. API y superficies

- [x] 3.1 Agregar `extraction.pdf_structure` a contratos, catálogo, CLI y MCP.
- [x] 3.2 Mostrar estructura simple/JSON en el lector o documentar lectura CLI.
- [x] 3.3 Actualizar contratos generados y pruebas.

## 4. Verificación y cierre

- [x] 4.1 Ejecutar Rust/CLI/MCP/UI/headless, fmt, Clippy, contratos y OpenSpec.
- [x] 4.2 Documentar guía/ADR y límites; conservar fallos.
- [x] 4.3 Sincronizar deltas y archivar; no iniciar fase6.
