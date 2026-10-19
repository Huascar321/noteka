# Investigación: parser PDF y límites

Consulta realizada el **13/10/2026**. Investigación documental/read-only, no ensayo del parser, medición local, instalación de dependencia ni prueba de empaquetado. Fuentes fijadas a versión cuando están disponibles; Context7 se contrastó con el tag publicado, no sólo con `main`.

**Actualización de apply, 14/10/2026:** dependencia fijada y auditada, helper/cola/
lector integrados en SQLite v9/49; [evidencia local](../../../../docs/verification-pdf-extraction.md).
La investigación original no se convierte retrospectivamente en benchmark: las
mediciones de implementación y sus límites están separadas.

El adapter y corpus propio evolucionaron de 207 a 235 casos, integrados al runtime.
El preflight evita `extract_text` y el decoder
permisivo de Flate; acceso tipado al mapping después de validar fonts. La revisión
de fuentes detectó además endobj opcional (`parser/mod.rs::_indirect_object`),
claves Dictionary duplicadas sobrescritas y `/Prev` cíclico cortado sin error
(`reader.rs::resolve_xref_and_trailer`). Se reprodujeron/corrigieron mediante
scanner léxico y segunda lectura de trailers/xref; el nuevo auditor rechaza ciclos
aunque el loader borre Prev. También se reprodujo el uso de miembros ObjStm sin
entrada xref: las referencias ahora validan índice vivo/generación. Codecs y
predictors lossless tienen implementación acotada; framing histórico/linearización,
precedencia free/hybrid y codecs de carga se contrastan con el corpus ampliado.
La vista sintética de carga conserva offsets originales tras auditarlos: no hay
reconstrucción ni aceptación de recovery. Ver guía/corpus y ADR 0012.

Referencias contrastadas para esta continuación: [ISO 32000-1 §7.4.4.4 y §7.5](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf)
y [xref/objetos comprimidos de qpdf](https://qpdf.readthedocs.io/en/stable/object-streams.html).
PNG 10–15 indica PNG al decoder; manda el tag por fila, no una coincidencia con el
Predictor del diccionario. Se corrigió una suposición contraria antes de fijar
el golden nuevo, con cinco positivos. qpdf documenta precedencia del suplemento
hybrid sobre su tabla y ambigüedad de Prev en el suplemento; no ignorar ambos.
`ObjectStream::new_with_limit` se contrastó con documentación y fuente 0.45.0:
se usa sobre bytes ya decodificados/validados para comparar el AST activo, no
como sustituto del scanner ni del decoder estricto.

## Candidates

| Candidato publicado a esta fecha | Licencia declarada | Hecho relevante | Decisión propuesta |
| --- | --- | --- | --- |
| [lopdf 0.45.0](https://crates.io/api/v1/crates/lopdf/0.45.0), 08/09/2026, MSRV 1.88 | MIT | Rust; carga desde bytes con opciones y extracción limitada por página. Default activa chrono-clock/rayon. | Fijar `=0.45.0`, `default-features=false`, sólo dentro del helper. Rust 1.99.0 existente supera el MSRV declarado, pero aún falta compilar. |
| [pdf-extract 0.12.1](https://docs.rs/pdf-extract/0.12.1/pdf_extract/), 16/09/2026 | MIT | Usa `lopdf ^0.42` y parsers adicionales de fuentes/CMaps. | No añade por sí solo aislamiento ni garantías de límites; evitar esa superficie extra para el subconjunto inicial. |
| [pdfium-render 0.9.4](https://docs.rs/pdfium-render/0.9.4/pdfium_render/), 06/09/2026 | MIT OR Apache-2.0 del wrapper | No incluye Pdfium: exige obtener/compilar/empaquetar la biblioteca C++ y verificar sus licencias/avisos por separado. | Diferir junto con render/fuentes/geométricas; no descargar un binario nativo ni adoptar todo el motor por este recorte. |
| [pdf_oxide 0.3.78](https://crates.io/api/v1/crates/pdf_oxide/0.3.78), 08/09/2026, MSRV 1.88 | MIT OR Apache-2.0 en crates.io | Alternativa Rust de alcance mayor. Las métricas publicitadas no se reprodujeron ni se auditó su contención. | No adjudicar calidad/rendimiento por marketing; no seleccionada en este cambio. |

Las licencias de la tabla son **de esas crates**, no una aprobación de todas sus dependencias. Apply deberá revisar el árbol resuelto, features, avisos/licencias transitivos y lockfile; no se incorpora un corpus de terceros sin procedencia/licencia.

## Verified APIs and caveats

- [`reader.rs` de v0.45.0](https://github.com/J-F-Liu/lopdf/blob/v0.45.0/src/reader.rs): `Document::load_mem_with_options` permite `strict` y `max_decompressed_size`. La carga puede recuperar xrefs u omitir problemas mediante logs; `strict=true` **no equivale** a una certificación de conformidad o ausencia de recuperación. El wrapper debe rechazar diagnósticos de degradación, no imprimir mensajes del parser que pueden incluir objetos/contenido.
- [`parser_aux.rs` de v0.45.0](https://github.com/J-F-Liu/lopdf/blob/v0.45.0/src/parser_aux.rs): `extract_text_with_limit`/`extract_text_chunks_with_limit` acotan descompresión de contenido de página y fuentes; el extractor no resuelve orden visual ni recorre Form XObjects. Las funciones sin límite no son aceptables aquí.
- [`document.rs` de v0.45.0](https://github.com/J-F-Liu/lopdf/blob/v0.45.0/src/document.rs): `was_encrypted()` distingue el cifrado original del estado actual. Una contraseña vacía puede causar decryption automática: `is_encrypted()==false` no basta para aceptar un archivo. Rechazar cifrado original y errores de contraseña; no solicitar credenciales.
- [`object.rs`](https://github.com/J-F-Liu/lopdf/blob/v0.45.0/src/object.rs) y [`encodings/mod.rs`](https://github.com/J-F-Liu/lopdf/blob/v0.45.0/src/encodings/mod.rs): errores de codificación pueden caer a StandardEncoding; glyphs sin mapping pueden omitirse y CMaps usar reemplazos. Una salida `Ok(String)` no demuestra ausencia de pérdida. El subconjunto inicial exige validar fuentes/operadores/bytes usados **antes** de extraer, rechazar mappings ausentes, y detectar logs de fallback sin guardarlos. No se usa `from_utf8_lossy` ni ToUnicode/encodings desconocidos.
- [RUSTSEC-2026-0187](https://rustsec.org/advisories/RUSTSEC-2026-0187.html): arrays/dictionaries profundamente anidados abortaban el proceso, sin posibilidad de recuperar con `catch_unwind`; el advisory declara parche `>=0.42.0`. 0.45.0 está fuera de ese rango afectado conocido, **no demuestra que carezca de otros fallos**. Mantener fixture sintética de anidación y verificar avisos actuales del árbol antes de fijar dependencia.
- [Sidecars Tauri 2](https://v2.tauri.app/develop/sidecar/): `bundle.externalBin` utiliza sufijo target-triple en el archivo preparado. Esto no acredita build/distribución de Noteka: hoy `bundle.active=false`, no hay helper y el host llama `execute_json` directamente. Se propone preparación local y resolver un sibling confiable en Rust; no añadir shell del frontend ni activar distribución.
- [Manual Apple getrlimit/setrlimit](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getrlimit.2.html): CPU y core dump son límites por proceso; RSS es una preferencia bajo presión, no un techo duro de memoria. No tratar `RLIMIT_RSS` ni tamaño comprimido del PDF como garantías de memoria. El plan usa **heap Rust vivo acotado por allocator del helper**, timer propio y supervisión; RSS completo se mide por separado. Esos mecanismos aún no están implementados/probados.

## Local evidence

- `crates/noteka-core/src/text_extractor.rs`: UTF-8 literal, NUL rechazado y ranges contra bytes originales; `extraction.rs::extraction_fragments` compara cada texto con el slice del adjunto. No reutilizar este DTO/lector para offsets PDF.
- `migrations/008.sql`: `covered_bytes=source_bytes`; worker suma ese campo para el presupuesto porque sólo hay texto literal. Mantener las tablas previas y la igualdad como cobertura de bytes fuente verificados, pero contar texto real de **ambos** extractores al admitir una publicación PDF.
- `extraction_worker.rs`: claim/publicación cortos y token/scope/revisiones/lease; el parser actual es en proceso con checks cooperativos. Un parser PDF que tarda/aborta no puede depender únicamente de esas callbacks.
- `workspace.rs` y `batch_storage.rs`: migraciones numeradas, listas de tablas/registry exacto, snapshots completos y configuración original en base v2. Tres tablas PDF aditivas necesitan actualizar integridad/registry/tests sin incluir sus filas en la huella original.
- `src-tauri/src/lib.rs`, `tauri.conf.json`, CLI y scripts headless: transporte genérico, sin invocación/empaquetado de helper existente. La prueba futura debe construir y lanzar el helper real desde CLI/direct-core y simular el layout del host sin abrir ventanas.

## Acceptance gates

1. Compilar el candidato fijado en Apple Silicon con el árbol real y licencias/avisos revisados.
2. Probar strings Standard/WinAnsi, recursos heredados y operadores Tj/TJ/'/", rechazando bytes no mapeados, fuentes no admitidas y fallbacks. Si la biblioteca pierde texto dentro del subconjunto, corregir un adapter acotado o revisar diseño; no expandir silenciosamente soporte.
3. Probar descompresión de contenido/xref/object streams, allocator bajo realloc/concurrencia y crash/abort/timeouts en un proceso real. El cap de heap no se publicita como cap de RSS.
4. Probar deadlines/pausa/cancelación y muerte del padre sin publicaciones tardías, helpers residentes ni core dumps; no atribuir al aislamiento de proceso una sandbox de permisos que no existe.
5. Medir release con corpus sintético reproducible, latencia/locks/RSS padre e hijo por separado. Los límites del [diseño](design.md) son criterios propuestos, no resultados.
