# Tasks

## 1. Independent projector

- [x] 1.1 Implementar biblioteca Rust estricta de geometría/filas/columnas/encabezado y conservación sin I/O ni integración; tests debug/release verifican golden independiente, huecos, orden, exteriores, cruces/ambigüedad, formato/ordinales y fronteras/excesos; documentar contrato y límites en docs/pdf-table-projector.md (local-extraction).
- [x] 1.2 Añadir ejemplo JSON acotado de desarrollo y pruebas de framing/errores sin emitir contenido en fallos; verificar roundtrip exacto y paquetes/lock existentes conservados, diferenciándolo de CLI/helper de producto (local-extraction).

## 2. Real evidence and resources

- [x] 2.1 Generar PDF propio MIT con golden de celdas previo a OCR, procesarlo explícitamente con el cliente vigente y contrastar proyección contra texto/celdas y ordinales reales, registrando errores/limitaciones sin normalizar ni fabricar OCR; verificar fuente/representación íntegras y guardar evidencia externa (local-extraction).
- [x] 2.2 Medir release con hasta512 regiones/512 celdas y representación grande,5 warmups/20 muestras, p95<=300ms y presupuestos/errores sin truncamiento; verificar toda evidencia referenciada exactamente una vez (local-extraction).

## 3. Integrated verification

- [x] 3.1 Ejecutar fmt/Clippy/tests del crate en ambos perfiles, contrastar dependencias/fuentes/20 binarios/catálogos16/API56 y Home sin cambios; mantener gates originales300/150 aprobados del producto sin alterar lectores o contratos (local-extraction).
- [x] 3.2 Revisar requisitos/diseño/diff, documentar resultados reales, actualizar plan/config y validar OpenSpec estricto; diferenciar componente independiente de integración de tablas aún pendiente (local-extraction).

## Workflow follow-up

- Sincronizar/contrastar sólo el delta verificado y archivar este cambio únicamente tras aceptación completa; nunca Home ni un cambio parcial por vencer la hora.
- Integrar después en un recorte separado con vigencia fuente/generación, operaciones/UI/FTS y recuperación si se persiste, antes de cerrar5a/avanzar5b.
- Detenerse antes de20:11:26UTC, con checkpoint/informe/prompt si queda trabajo.
