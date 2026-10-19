# Design

## Context

Baseline durable: SQLite v12, API v1/52, MCP 24/44/45 y nueve mutaciones batchables. El lector aislado macOS reconoce PNG/JPEG con Vision r3/fast, es-ES/en-US; no está conectado a persistencia. La UI ya separa preparación, lectura y búsqueda. Home permanece 9/17, fuera de este cambio. Ver ADR 0019 y evidencia externa indicada en `docs/verification-ocr-reader.md`.

## Goals / Non-Goals

Integrar sólo OCR de imágenes administradas y presentar su evidencia inferida mediante las operaciones compartidas. Conservar límites, originales, fencing, validación completa y recuperación. No OCR de PDF, reconstrucción de tablas, embeddings, agente, proveedor remoto, modelos descargados ni modificaciones de Home. No inferencia al leer, abrir, buscar, migrar o restaurar.

## Decisions

### Protocolo decimal privado v2

El sondeo sintético de 512 regiones encontró 135 diferencias de bits al parsear números JSON con la configuración actual de serde_json. No demuestra corrupción persistida, pero impide depender de ese codec para una representación canónica nueva. No activar `float_roundtrip` globalmente: afectaría datos desconocidos y hashes ajenos.

Mantener `NKOCR001`/v1 intacto. Añadir `NKOCR002`/v2: Swift emite `String(Double)` para confianza y bbox; Rust usa `str::parse::<f64>()`, valida finitud/rango/geometría y normaliza una vez con `to_string()`. Persistir y publicar cadenas canónicas <=512 bytes, incluyendo subnormales y cero negativo. Perfil del motor sin cambios: v2 versiona transporte, no calidad. Validar caps antes de normalizar arrays/serializar; representación expandida >1 MiB se rechaza completa. Tipos públicos Ocr* evitan colisiones de schemas y preservan Eq de localizadores existentes.

### Persistencia y migración antes de ejecución

ADR 0020 registra el formato antes de editar DDL. Migración 013 agrega `extraction_ocr_generations` y `extraction_ocr_chunks`, con ownership compuesto y cascadas coherentes con generaciones. No editar 001–012. Representación versionada canónica con imagen, motor/OS, espacio de coordenadas, limitaciones y regiones; SHA-256 y partición UTF-8 de <=4096 bytes, <=512 chunks/1 MiB. `fragment_count` del job cuenta chunks internos, no regiones; `covered_bytes` describe procesamiento de la imagen completa, no exactitud de transcripción. OCR vacío sigue siendo generación válida y legible.

Retención compartida <=32 MiB, descontando sólo la generación reemplazada del mismo job. Lectura/restore/index verifican ownership, forma, conteos, bytes/hash, partición, fuente y ausencia de representaciones cruzadas. Corrupción devuelve error; nunca purga/repara.

Apertura v12: backup consistente y verificable antes de transacción, 013 y versiones atómicos, integridad antes de commit. Fixtures v12 se crean con las copias independientes del baseline antes de cambiar el código. Probar rollback, restore del backup con cliente v12 en ruta nueva y rechazo de v13 por cliente antiguo antes de escrituras. Versión futura v14 se rechaza sin modificar archivos. Snapshots completos incluyen tablas OCR y límite 64 MiB; `base.v2` sigue excluyendo derivados. Recibos antiguos invalidan por versión. Configuraciones futuras se conservan sin ejecutar ni reinterpretar.

### Ejecución y lectura

`image_ocr` extractor/config v1 reutiliza enqueue/run/cancel/retry/control. Admisión PNG/JPG/JPEG y MIME compatibles, fuente <=1 MiB. El helper verifica firma real, una imagen, <=4096 por eje/4 Mp y orientación. Fuente se verifica antes de reclamar/publicar; ejecución fuera del writer lock, lease/fence existentes y publicación atómica. Preservar generación anterior ante fallo/cancelación/exceso. Supervisor con pipes acotados no bloqueantes, timeout <=5 s, kill/reap y entorno/CWD aislados. Native RSS es observado a intervalos, no límite duro ni garantía GPU; declarar esa limitación.

`extraction.ocr_regions`: job, limit 1–64/default32, cursor ligado a operación/workspace/job/generación/limit. Verificar representación y fuente completas antes de paginar. Respuesta con generación, fuente, manifiesto y regiones literales (incluye bbox/confianza decimales). No helper en lector o recuperación. Datos futuros se conservan incompatibles; una ejecución sin helper falla explícitamente.

### Proyección y contratos

Selección de generación independiente mediante `extraction.index`. Proyectar sólo texto reconocido de cada región, sin texto de metadata/JSON; dividir UTF-8 en <=4096 bytes, <=512 fragmentos/1 MiB. Localizador `image_ocr_region_text` con ordinal, bbox/confianza, espacio, bytes de región y rango derivado. Hash de contenido incluye representación/proyección completa, fuente y generación. Mantener bytes de hash, perfiles y comportamiento UTF-8/PDF/XLSX. Cuotas globales de índice sin cambios (1024 entradas, 16384 fragmentos, 8 MiB). Validación completa de candidatos/postings/fuente/copia, sin cache de confianza.

API v1/53: añadir sólo el lector; ampliar enums/localizadores nombrados de index/search y metadata de capacidades. Comparar los 52 contratos del baseline: otros 50 idénticos. MCP allowlist read agrega lector: 25/45/46; siete exclusiones y nueve mutaciones batchables intactas.

### Interfaz y distribución

Agregar cuarto tipo a preparación/lector y un mapeo explícito de operaciones: nunca fallback OCR→XLSX. Lectura UI de 20 regiones y manifiesto completo; texto inferido, limitaciones, procedencia, revisión y errores visibles. Mostrar bbox/confianza como metadata, sin promesa de exactitud. Search individual/agrupada muestran localizador OCR; mismo orden/grupos/cursor. Mantener guardias de buffers, lifetime y máximo físico de una llamada de preparación/lectura, incluida respuesta tardía. No llamadas o mutaciones al montar/abrir/buscar.

Preparar binario Swift para target/perfil macOS usando SDK instalado y overlay Tauri macOS si corresponde; no agregar helper macOS al build de otros sistemas. No afirmar disponibilidad por catálogo estático. Build no-bundle headless no prueba IPC/foco/cierre nativo.

## Risks / Trade-offs

- OCR del sistema varía por OS; persistir versión/build, no prometer lectura perfecta ni orden visual. Corpus humano y CER/WER separado de oráculos de persistencia.
- Decimal canónico ocupa más que notación científica; rechazo íntegro por cuota, sin truncar.
- Validación completa puede afectar gates; medir corpus original sin reducirlo, investigar cualquier fallo sin relajar 300/150 ms.
- Versiones antiguas rechazan v13: recuperación sólo mediante backup en otra ruta, no downgrade.

## Verification

Pruebas v1/v2 y redondeo bit a bit, entradas malformadas/caps; integración nativa real con fuente sintética y captura del resultado real para oráculo independiente, sin presentar fake como inferencia. Rust en ambos perfiles, oráculos PDF/XLSX previos completos, migración/backup/restore/snapshots, fences/concurrencia/corrupción/CAS y cuotas compartidas. CLI/MCP raw+SDK y catálogos; UI headless ambos temas con errores/cursores/buffers/interleavings y cero tráfico externo. Cuatro gates finales: dos mixtos y dos UTF-8/PDF sin modificar corpus/objetivos. Evidencia, hashes y fallos fuera de Git; revisar código/specs/diseño antes de sincronizar y archivar sólo este cambio.

## Open Questions

Ninguna decisión de alcance pendiente. La disponibilidad del motor fuera del macOS probado se expresa como incompatibilidad, no como capacidad verificada.
