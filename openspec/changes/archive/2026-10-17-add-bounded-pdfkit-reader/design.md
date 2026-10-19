# Design

## Context

Ver propuesta y evidencia externa en active-pdfkit-reader-path.txt. Baseline durable522 archivos/16 binarios, SQLite13/API53/MCP25/45/46/batch9. El lector PDF anterior y su protocolo quedan intactos. Los sondeos propios recuperan una línea Unicode que el anterior rechaza; no prueban PDF genérico. Main todavía tiene tres límites textuales heredados anteriores a las integraciones verificadas; el delta conserva sus escenarios y los refiere a los contratos vigentes.

## Goals / Non-Goals

Objetivo: un perfil privado independiente, con fidelidad delimitada y proceso acotado, que permita evaluar texto PDF nativo antes de diseñar almacenamiento. Rust valida toda respuesta. Swift sólo lee bytes mediante el framework y no administra dominio/estado.

No hay vínculo al core, nuevas operaciones, tabla, selección, lector UI o staging Tauri. No OCR/raster/tablas/geometría, descripción visual o modelo. Home9/17 y sus criterios nativos permanecen fuera del cambio.

## Decisions

### Perfil de texto del sistema

Usar PDFKit del SDK existente con perfil `apple_pdfkit_text_v1`. Fuente1MiB,1–100 páginas y1MiB de texto UTF-8 total. Framing propio: cabecera PDF admitida y marcador %%EOF final, con whitespace PDF permitido. Es un preflight acotado, no auditoría completa de xref/objetos; el backend puede recuperar anomalías internas. El lector estricto previo sigue siendo otra opción, sin fallback entre perfiles.

Rechazar documento no reconocido, página ausente y cualquier isEncrypted/isLocked verdadero antes de pedir texto. Apple intenta internamente contraseña vacía al abrir ciertos documentos: no afirmar que nunca lo intenta; el helper no llama unlock ni acepta contraseñas. Se rechaza también un cifrado ya desbloqueado.

### Texto y procedencia sin pérdida propia

Usar PDFPage.attributedString como objeto y extraer CFString sin puente previo a String. CFStringGetBytes UTF8 con lossByte0 debe convertir todas las unidades UTF16; comprobar longitud/tamaño antes de asignar buffer y rechazar NUL o conversión parcial. No normalizar ni sustituir caracteres. Esto preserva el resultado del API, no certifica que PDFKit reproduzca perfectamente el texto fuente.

Las páginas mantienen número físico 1-based, estado text/no_text y texto completo; vacío/nil no demuestra ausencia visual. Document v1 registra conteos/bytes exactos, API real, OS/build y limitaciones fijas. No devuelve nombres de nota, bbox, confianza, fuente tipográfica o metadata que no consultó. Cinco limitaciones: parser no valida integridad PDF completa, orden de selección no equivale a visual, sin OCR/tablas/geometría, no_text no prueba ausencia visual y resultado dependiente de OS.

El sondeo mostró diagnósticos nativos de attributedString aun con PDF válido. La ruta normal redirige stderr del framework a /dev/null antes de invocarlo; fallos de comprobaciones producen razones tipadas. No expone texto/diagnósticos al usuario ni amplía el pipe8KiB. Los probes emiten deliberadamente por pipes antes de esa ruta y sólo existen en su build separado.

### Transporte privado propio

Paquete `noteka-pdfkit-protocol`, sin dependencia desde el núcleo. Header24B/magic NKPDK001/version1, enteros little-endian source_bytes/remaining_ms y reservados cero. Header ajeno no se reinterpreta. Respuesta JSON versionada con perfil fijo, SHA-256 fuente, documento y razón enum; todos los objetos rechazan campos desconocidos. Metadatos se acotan antes de split/collect. Páginas consecutivas y estados/bytes/conteos completos, texto1MiB, stdout8MiB/stderr8KiB; nada de floats o features JSON globales nuevas.

No elegir aquí un formato persistente. Ese diseño requiere otro cambio y backup/migración/restore antes de DDL. Los originales siguen siendo verdad; el derivado sólo conserva texto del perfil elegido.

### Contención y preparación

Helper Swift descartable, stdin únicamente, sin rutas de origen. RLIMIT_CORE0/CPU4s, watchdog monotónico5s desde inicio, plazo sólo acortable. Máximo RSS observado128MiB@5ms y al publicar, no límite duro RSS/GPU ni allocator Rust. El consumidor acota pipes, timeout, termina/recolecta sólo su proceso/grupo; CWD temporal y entorno aislado. No queda servicio residente.

Preparación con SDK/Swift instalados, target aarch64/x86_64 macOS13+ y perfil debug/release. No instalar componentes. Normal y probes tienen nombres propios; los flags de pruebas son rechazados por normal. Nada se añade a hooks/staging del producto en este recorte.

### Evidencia y regresión

Corpus de fuentes propias con expectativas humanas fijadas antes del lector, fuentes comunes/Unicode/rotaciones/páginas, vacíos/imágenes/cifrado, framing y cuotas; expectativas limpias exactas por página y límites de layout declarados. No usar salida PDFKit como golden ni retocarlo tras fallar. Corpus malformado documenta rechazo de preflight/backend, sin prometer detectar todos los PDF corruptos.

Rust debug/release, corpus/procesos por perfil, respuesta/metadata corruptas y probes de UTF16/recursos. Preservar522 originales y16 builds durables, comparar los53 contratos completos. Cualquier diferencia de build se investiga y etiqueta: restaurar una copia no prueba rebuild idéntico. Tauri copia staging compartido a cada perfil; preparar su entrada correcta o aislar su build antes de fijar artefactos. No construir host entre fijación y gates.

Los cuatro gates recién aprobados corresponden a los builds previos medidos. Si fuente o artefacto del producto cambia materialmente, repetir checks/gates pertinentes con corpus/objetivos intactos; si siguen idénticos, conservar la atribución original sin anunciar una ejecución nueva. UI no cambió: sus3442 checks anteriores no se cuentan como nuevos.

## Risks / Trade-offs

- Parser de sistema tolerante y dependiente del OS → perfil/OS/build y limitaciones explícitos; nunca reemplazar pdf_text por defecto.
- Memoria/tiempo nativos variables → límites observados y abort completo, no ajuste del target para admitir un caso.
- Unicode/orden pueden variar dentro del API → conversión propia estricta, corpus independiente y ausencia de promesa visual.
- Tres literales normativos quedaron atrás de integraciones → reconciliar sólo esos bloques completos con core-operations/cuotas reales; archivos históricos intactos.

## Migration Plan

No modifica formatos persistentes. Desplegar sólo el helper privado y su validador; revertir su uso no implica migrar ni tocar SQLite/originales. Mantener baseline/copias y evidencia fuera de Git. Integración posterior exige decisión de almacenamiento/recuperación separada antes de cambios persistentes.

## Open Questions

Ninguna decisión material pendiente. Otras versiones de OS/arquitecturas se declaran no verificadas hasta medirlas.
