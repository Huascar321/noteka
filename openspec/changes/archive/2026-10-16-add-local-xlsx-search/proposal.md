# Proposal

## Why

Al proponer el cambio, los XLSX administrados ya se extraían y leían sin pérdida, pero no se podían buscar:
`extraction.index` los rechazaba explícitamente. Este recorte de 5a conecta
esos derivados verificados con la recuperación textual existente, antes de añadir
más extractores o instalar modelos.

## What Changes

- Admitir selección explícita de `xlsx_cells` v1 con `extraction.index put` y
  búsqueda/get filtrados por ese extractor, sin nuevos comandos u operaciones.
- Proyectar valores guardados tipados y texto de fórmulas como campos separados,
  con números léxicos y rangos UTF-8 hoja/A1/celda/campo. No evaluar fórmulas,
  expandir shared/array, inferir encabezados ni buscar el JSON de almacenamiento.
- Mantener originales y Workbook canónico como autoridad; reconstruir/verificar
  íntegramente campos, copias, postings y candidatos con las fronteras actuales
  de verificación de fuentes. Sin parsing, helpers o ejecución al indexar/buscar.
- Conservar caps: 512 fragmentos/1 MiB por selección, 1.024 selecciones,
  16.384 fragmentos/8 MiB de texto agregado, snapshot 64 MiB y respuesta 2 MiB.
  Dividir campos largos sin cortar UTF-8, rechazar exceso completo, nunca truncar.
- **BREAKING:** migración de compatibilidad v11→v12 con backup/rollback. No cambia
  tablas/FTS ni reindexa lo anterior; impide que un cliente v11 interprete o borre
  selecciones XLSX que no reconoce. Recuperar software previo requiere su backup.
- **BREAKING para clientes de enums cerrados:** ampliar sólo los DTOs del índice
  y búsqueda con extractor/localizador XLSX; no prometer esquemas anteriores
  idénticos en esas dos operaciones. API v1 conserva 52 operaciones, MCP
  24/44/45, nueve mutaciones batchables y los otros 50 contratos.
- Mantener perfil/tokenizador/ranking y hashes UTF-8/PDF. Repetir dos gates FTS
  intactos <=300 ms y añadir gate mixto XLSX con el mismo objetivo, validación
  completa y mediciones de locks/memoria, sin aumentar límites.

## Capabilities

### New Capabilities

Ninguna; se extienden las fronteras ya existentes.

### Modified Capabilities

- `local-text-search`: selección XLSX, proyección localizada, cobertura e
  integridad de un índice mixto.
- `local-extraction`: reemplazar el veto absoluto por indexación sólo explícita,
  sin alterar la extracción ni su reader.
- `local-workspace`: v12 de compatibilidad, conservación de índices anteriores,
  backup/restauración y rechazo de versiones futuras.
- `core-operations`: extensiones de enums documentadas y contrato/catálogo
  compartido, sin nueva operación o mutación batchable.
- `local-mcp`: búsqueda/indexación XLSX bajo permisos vigentes y admisión v12.

## Impact

Contratos, proyección canónica, coverage y validación del índice Rust; migración
012/registry, contratos generados y pruebas CLI/MCP. No nuevas dependencias.
Baseline independiente v11/52: 369 archivos, ocho binarios y catálogo fuera de Git.
Adenda 16/10: esos artefactos temporales se perdieron. El usuario autorizó repetir
la aceptación con evidencia durable y límites idénticos, documentando que la
comparación exacta contra ese baseline ya no puede demostrarse nuevamente.
Ver [criterio alternativo](../../../../docs/xlsx-search-evidence-recovery.md).
Las decisiones de proyección/v12 fueron propuestas técnicas adoptadas y verificadas; el usuario autorizó
continuar el desarrollo local, no remotos o archivos personales. No hay dependencia
externa bloqueante conocida; rendimiento/compatibilidad requieren aceptación real.

**Cierre 16/10:** 19/19 tareas, cinco deltas sincronizados/contrastados y cambio
archivado; [evidencia durable](../../../../docs/verification-xlsx-search.md).
287 Rust por perfil/606 checks headless, dos gates mixtos 167,02–270,74 ms y dos
intactos 114,31–152,19 ms p95, sin rebajar caps/targets/validación. Comparaciones
históricas v11 no repetibles según autorización; Home sigue activo 9/17 aparte.

## Non-goals

No UI/Home, encabezados inferidos, cálculos/tablas relacionales de Excel, macros,
PDF amplio/OCR, búsqueda semántica, modelos, OAuth, proveedores o publicación.
El recorte no completa 5a ni satisface por sí solo todas las dependencias de 5b.
