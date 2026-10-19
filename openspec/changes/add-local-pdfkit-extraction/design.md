# Design

## Context

Baseline durable542 archivos/22 builds, SQLite13/API53/MCP25/45/46/batch9. Lector
PDFKit cerrado6/6; conserva Document v1, páginas físicas y limitaciones reales.
Hoy no hay extractor pdfkit_text, persistencia o operación de lectura del formato.
Home9/17 y sus ocho aceptaciones nativas no participan. Evidencia externa indicada
por active-pdfkit-extraction-path.txt; no se reinicia el deadline02:16UTC de5a.

## Goals / Non-Goals

Integrar preparación y lectura backend, por operaciones compartidas y acción
explícita. Mantener PDF estricto anterior, todos los contratos previos, cuotas,
validación, revisiones y cercas. No FTS/UI de este formato, OCR/raster/tablas,
ampliaciones Excel, modelos, proveedores o pantalla. No anunciar integración por
tener únicamente el runtime/propuesta; las tareas posteriores deben completarse.

## Decisions

### Consumidor Rust antes de persistencia

Paquete privado noteka-pdfkit-runtime, independiente de SQLite/núcleo, con función
de bytes, helper explícito y presupuesto1..5000ms. Retorna Document validado o
errores tipados; conserva un error genérico de la cerca del caller sin reformularlo.
Cada llamada comprueba cerca antes de spawn, durante polling y antes/después de
decodificar. Deadline monotónico propio no puede superar el presupuesto restante
del claim. Pipes no bloqueantes8MiB/8KiB, stderr no vacío rechaza; source1MiB y
SHA esperado. Recolecta únicamente su Child en Drop, sin tocar otros procesos.

macOS usa sandbox-exec deny network para el hijo, CWD temporal y entorno mínimo.
El sondeo local de sandbox anidado pasó; se conserva esa guardia también cuando
el consumidor se ejecuta bajo el sandbox externo de aceptación.
El helper ya controla CPU4s/vida5s/RSS observado128MiB@5ms. El padre conserva su
propio plazo/cancelación aunque el helper no coopere. Razones de memoria observada,
deadline, setup, salida fallida, protocolo o rechazo tipado no incluyen fuente.
No atribuir SIGKILL genérico a CPU. Plataformas no disponibles fallan explícitamente.

### Perfil y representación

Extractor pdfkit_text versión1/config v1 chunk4096, distinto de pdf_text. Conservar
Document privado v1 como JSON canónico de structs con orden de campos fijo, sin
floats, normalización o sustitución. Máximo1MiB de representación serializada,
chunks UTF-8 <=4096/512; el cap de texto del helper no garantiza que cualquier
texto1MiB quepa en ese JSON. Un exceso falla completo antes de publicar.

Manifest público propio PdfkitManifest/PdfkitEngine, con perfil, hash/tamaño de
representación, páginas/text_bytes, procedencia y limitaciones; nombres de schema
propios para no renombrar definiciones OCR anteriores. La única representación
persistida incluye todos los textos y metadatos. No duplicar texto íntegro en un
manifest SQL o contar sólo texto visible al sumar retención.

### Recuperación diseñada antes de DDL

Crear fixtures reales con CLI13 durable antes de escribir014: vacía y mixta con
derivados anteriores y backup consistente mediante operación antigua. Registrar
schema/filas, originales/hashes, revisiones, control, ownership y contratos53.
No etiquetar fixture SQL construida con el cliente nuevo como baseline histórico.

Migración014 aditiva: extraction_pdfkit_generations y extraction_pdfkit_chunks,
FK compuesta job/generation con cascada, tamaños/counts/ordinales SQL acotados.
Generation registra version/profile/hash/bytes/page_count/text_bytes/chunk_count;
chunks conservan JSON una sola vez. Actualizar versión14 en la misma transacción,
backup13 previo consistente. Migraciones001–013 byte idénticas. Preflight sólo
consulta tablas nativas cuando versión>=14. Registro de schema/snapshot incluye
ambas; ORIGINAL_TABLES/base_hash v2 no cambia, retención agrega JSON nativo a32MiB.

Antes de admitir lectura/publicación: contar y acotar en SQL antes de materializar,
comprobar ownership/perfil/hash/configuración/cruces entre formatos; verificar
JSON canónico completo y fuente íntegra. Revalidar dentro del writer lock además
de fuera. Regeneración descuenta sólo su generación reemplazable, sin purga.

Probar fallos antes/después de marcar versión: rollback atómico a13. Cliente13
rechaza14 sin writes; cliente14 rechaza15. Restaurar backup13 con cliente antiguo
en nueva ruta y comprobar fuentes/datos. Añadir casos de configuración/documento
desconocidos sin borrarlos ni normalizarlos al migrar. Ningún helper se ejecuta
al migrar/restaurar; instrumentar conteo de launches. No downgrades in-place.

### Cola y lectura compartidas

Integrar tipo explícito en elegibilidad/known_config/claim/publicación/cancelación
sin cambiar estados, leases, source/hash/revisión o whitelist. Lectura
extraction.pdfkit_pages con id/limit1..64/default32/cursor; páginas completas con
número físico, estado/texto y manifest/coverage. Cursor liga job/generation/límite.
Sin helper para leer o reabrir. Tipos ajenos y cursores viejos fallan cerrados.

FTS rechaza esta generación de forma explícita hasta otro cambio; no convertirla
en PDF estricto/UTF8 ni reinterpretar chunks JSON como contenido buscable. La UI
existente no anuncia el formato nuevo en este recorte.

### Transportes y empaquetado

Nueva lectura54, conservar53 inputs/outputs previos y nueve mutaciones batchables;
sólo añadir schemas propios y metadatos de capacidad identificados. CLI/MCP usan
el núcleo y no reciben rutas de helper, comandos o contraseñas. MCP26/46/47,
lectura por defecto; run sigue exigiendo allow-write+allow-run. Siete exclusiones
sin cambios. Generar bindings desde catálogo sólo después de implementar operación.

Preparar staging/externals del nuevo helper explícitamente para el host, sin
confundir builds de staging con los realmente medidos. Preservar variantes y
fijar artefactos antes de gates; no reconstruir host entre pins y mediciones.

## Risks / Trade-offs

- Más JSON que texto por escapes → cap canónico independiente, error completo.
- Backend dependiente del OS → perfil/OS/build/limitaciones conservados.
- Nueva tabla derivada afecta snapshots/cuota → fixtures13 reales, registry y
  validación completa antes de publicar; prueba mixta real de32MiB.
- Paso parcial al deadline → dejar sólo tareas verificadas marcadas, cambio activo
  sin sync/archivo, fuentes/baseline/evidencia durables; informar qué falta.

## Migration Plan

Orden obligatorio: baseline + diseño → runtime Rust y fixtures13 probados antes de014 →014 y
storage → cola/lectura → transportes/packaging → regresión/gates → sync/archivo.
No crear DDL antes de guardar fixtures y expectativas de recuperación. No ejecutar
una migración sobre datos personales durante desarrollo; sólo workspaces sintéticos
externos. SQLite/originales siguen siendo verdad. Backup restaura13 en otra ruta.

## Open Questions

Ninguna decisión material pendiente. UI/FTS se entregan por cambios independientes
y este backend no completa5a; no saltar a5b/5c/6 por cerrar este recorte.
