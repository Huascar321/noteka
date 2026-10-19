# Design

## Context

Ver [propuesta](proposal.md). `extraction_worker.rs:234` despacha UTF-8/PDF; `extraction_storage.rs:498` y el índice de adjuntos asumen esos dos formatos. Integrar allí un parser no verificado mezclaría decisiones de lectura con migraciones y nuevos contratos. El baseline externo de 326 archivos y seis binarios está en el directorio de auditoría, fuera de Git.

## Goals / Non-Goals

**Goals:** validar el lector real y su proceso descartable sin tocar persistencia. El protocolo no es una operación pública ni recibe nombres de archivo. La salida estructurada preserva lo leído, no la presentación visual.

**Non-Goals:** ver propuesta; en particular no se crea una tabla provisional ni se amplía el enum de localizadores FTS. La integración siguiente debe diseñar recuperación/migración antes de modificar v10.

## Decisions

1. **ZIP =9.0.3 + roxmltree =0.21.1**, sin features ZIP default y sólo Deflate con backend Rust. Se revisan versiones, API, licencias y árbol reales. Calamine =0.36.1 ofrece streaming pero `DataRef::Float(f64)` no preserva lexemas; no se usa como fuente de verdad numérica. quick-xml es alternativa válida, pero DOM acotado simplifica relaciones/namespaces y validación estructural; el parser XML no se reimplementa.
2. **Preflight ZIP antes de ZipArchive:** verificar EOCD/central/local, cantidades/tamaños/offsets, nombres ASCII canónicos sin traversal, alias o duplicados, flags/métodos y descriptor de datos. Rechazar ZIP64/multidisco, prefijos y bytes extra ambiguos. Leer con CRC y límite+1 todas las entradas en memoria, nunca a disco. Validar todas las relaciones, no sólo las usadas, rechazando externas. Las partes opcionales no transcritas se declaran como limitaciones; macros/conexiones/alternate content se rechazan.
3. **OOXML Transitional exclusivamente:** resolver workbook desde `_rels/.rels`; targets internos canónicos absolutos o relativos sin `..`. Validar content types y relaciones de worksheets/shared strings/styles. No confiar en `dimension` para reservar memoria; filas/celdas deben estar ordenadas con referencia explícita. XML UTF-8, sin DTD ni resolver, nodos/profundidad acotados. Rich text concatena sólo runs textuales; formato/phonetics no se interpretan y contenido textual alternativo no se omite silenciosamente.
4. **DTO independiente:** workbook con sistema 1900/1904, estilos numéricos, hojas y celdas. Celda: fila/columna/A1/estilo, caché opcional tipada, fórmula literal y metadata. Números son strings con gramática decimal/exponente finita; nunca f64. Rango declarado y observado separados; merges no expanden grillas. Fórmulas shared preservan ancla/índice/rango y seguidores, no se expanden. Array retiene rango. No se infieren encabezados ni se recalculan caches.
5. **Límites adicionales:** 512 estilos/códigos numéricos, 10.000 shared strings, 1.000 merges por hoja, nombre de hoja 31 caracteres; coordenadas Excel 1–1.048.576 / 1–16.384. Representación JSON completa de workbook <=1 MiB. Protocolo binario de 24 bytes con magic/versión/tamaño/deadline; una respuesta JSON estricta con versión, parser y hash fuente, sin rutas/logs sensibles. Recibe EOF después del original. Validación de salida reutilizable por el futuro supervisor.
6. **Runtime común:** extraer allocator/watchdog/rlimits de PDF sin cambiar algoritmos/límites; ambos helpers lo usan. El watchdog arranca antes de stdin; 128 MiB son Layout vivos solicitados, no garantía RSS ni sandbox. La red se deniega adicionalmente en el harness y se comprueba cwd temporal sin escrituras; no se afirma un sandbox FS. El binario no ejecuta macros, shell ni accesos externos. La integración futura deberá supervisar/recoger al proceso y cercar publicación.

## Risks / Trade-offs

- [Subconjunto rechaza XLSX válidos] → motivos explícitos y corpus positivo/negativo; no se anuncia Excel genérico.
- [DOM/ZIP consumen memoria antes de contar celdas] → preflight + tamaños reales + nodos/profundidad + allocator separado.
- [Caché obsoleta/estilos complejos] → valor guardado no equivale a recálculo; se preservan códigos, no se renderizan.
- [Runtime compartido puede afectar PDF] → pruebas originales intactas y corpus/contención en debug/release.
- [Corpus sintético no demuestra compatibilidad universal] → golden independiente y limitaciones documentadas; ningún dato personal.

## Migration Plan

No cambia formato persistente ni hay migración. Se añaden binarios privados de desarrollo, no se empaquetan en Tauri ni se activan desde core. Retirar esta entrega no cambia workspaces existentes. El siguiente cambio añadirá selección explícita, lector público, validación persistida y migración aditiva con backup/restore, sin asumir que esta entrega ya lo hace.
