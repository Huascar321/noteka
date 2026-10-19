# Revisión del lector PDFKit independiente

Revisión local de implementación, diseño, requisitos y evidencia. Alcance:
add-bounded-pdfkit-reader; no equivale a revisión manual de UX o aceptación nativa.

## Resultado actual

Implementación, contención y cuatro gates finales consecutivos aprobados. Revisión
de código/diseño/requisitos completa, sin hallazgos abiertos de implementación.
Único delta sincronizado/contrastado y cambio archivado como2026-10-17-add-bounded-pdfkit-reader.

## Revisión de implementación

- El núcleo no depende del paquete nuevo; sólo Cargo.toml/Cargo.lock anteriores
  cambiaron para añadir un miembro/paquete. Ninguna entrada anterior del lock se
  modificó. Fuentes previas preservadas, contratos53 completos iguales y copias
  durables de los16 artefactos previos. No cambia SQLite13 o whitelist batch9.
- Cabecera propia y enums/versiones/perfil estrictos, hash ligado al esperado,
  deny_unknown_fields en envelopes y objetos, metadatos acotados antes de split,
  conteos completos y páginas consecutivas. Ninguna feature JSON global nueva.
- CFString se convierte sin puente String anterior, comprobando conversión total,
  bytes antes de asignar buffer y NUL. Se conserva el resultado del API; no se
  promete fidelidad del fuente PDF o normalización/orden visual. Autoreleasepool
  por página y guardias de RSS/plazo antes de publicar.
- PDFDocument de bytes, framing mínimo explícito, cifrado reconocido rechazado
  antes del texto y ningún unlock. Enlaces/instrucciones permanecen datos. No se
  invocan acciones, OCR, rasterización, red, modelos, jobs o workspace.
- CPU/core dumps se fijan antes de leer; deadline monotónico sólo se acorta. RSS
  se muestrea cada5ms y al emitir, sin presentarlo como límite duro RSS/GPU.
  Canal exclusivo conserva stdout del protocolo y descarta diagnósticos nativos.
- Probes separados, normal rechaza todos sus flags. Consumidor de aceptación con
  límites de pipes, CWD/entorno temporal y terminación/recolección sólo de su
  grupo; prueba de padre efímero conserva/vigila únicamente su PID y evita matar
  uno ya desaparecido. No hay proceso residente ni staging automático del host.
- Preparador usa SDK existente y cargo metadata offline; target/opciones se
  validan y destino symlink se rechaza conservando el marcador original. Sólo
  macOS27.0.1/aarch64 tiene ejecución verificada; macOS13+ es target de compilación,
  no una matriz de compatibilidad ya aceptada.

## Evidencia y límites

Ver verification-pdfkit-reader.md y la evidencia externa.59 casos únicos nativos
por perfil:22 archivos,13 wires,24 procesos; siete tests privados y326 de regresión
Rust por perfil. El corpus no usa PDFKit para generar expectativas. El PDF de
texto rasterizado da no_text, sin convertirlo en ausencia visual. Corpus v1 y v2
permanecen; no se corrigieron goldens para ajustarlos al lector.

Dos gates mixtos fallaron en consultas distintas, con locks aprobados. Hay carga
concurrente observada y diferencias entre CPU/wall en diagnóstico A/B, pero la
causa no se demostró. Se preservan fallos, sin optimización o excepción de target.
No se confunde el diagnóstico CLI con un gate aprobado. No se modificó ni detuvo
el laboratorio de otro proyecto. Los checks UI previos siguen atribuidos a su
cierre, sin inventar una repetición. El límite horario de5a permanece fijo.

## Cierre técnico

Los cuatro gates consecutivos de retry3 pasaron: query p95 máximo267,457625 ms y
lock22,331125 ms, objetivos300/150 intactos. Fallos anteriores conservados, causa
no demostrada. Plan/guías/ADR reflejan este resultado. Sólo este delta participa
en sincronización/contraste y archivo; Home9/17 permanece activo y separado.

Integración del perfil PDFKit, OCR de PDF y ampliaciones Excel son cambios futuros.
No se declara toda5a,5b,5c o6 terminada con el lector privado.
