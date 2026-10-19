# Revisión técnica de OCR integrado — aprobada

Revisión propia de código, diseño, contratos y escenarios de
`add-local-image-ocr`. No es una revisión externa ni aceptación manual de UX.
La regresión debug/release y los cuatro gates heredados están aprobados. Ver [verificación](verification-image-ocr.md) y
[matriz de escenarios](verification-image-ocr-scenarios.md).

## Persistencia y recuperación

La migración 013 agrega dos tablas con ownership compuesto y cascadas; conserva
001–012. El ADR 0020 se escribió antes del DDL. La representación versionada usa
JSON canónico dividido en chunks UTF-8 de hasta 4096 bytes, con hash, conteos y
presupuestos comprobados antes de materializar el texto de SQLite. Ownership de
chunks y perfil se comparan en SQL para no cargar metadatos hostiles sin límite.
Lectura, restore e índice revalidan forma, partición, hash y cruces de formato.
La fuente se verifica antes de paginar; no se ejecuta OCR para recuperarla.

Los fixtures v12 fueron creados con binarios independientes del baseline durable
antes de editar el formato. Las migraciones reales de ambos perfiles conservan
las 37 tablas previas y agregan dos. El primer intento detectó una consulta a la
tabla nueva durante preflight v12; quedó corregido mediante la versión efectiva
del workspace y cubierto por regresión específica. No hubo writes en ese intento.
La recuperación usa un destino nuevo: no hay downgrade sobre el original.

## Ejecución y contratos

OCR reutiliza claim, lease, fences, cancel/retry, presupuesto de ejecución y
publicación atómica. El helper trabaja fuera del writer lock, con entorno y CWD
aislados, pipes acotados, timeout, kill y reap. La revisión incorporó una segunda
comprobación de formato/metadata dentro de la transacción de publicación. La
prueba nativa alcanza los hooks y comprueba preservación de la generación previa.
La retención de 32 MiB se comparte con los otros tres extractores; sólo descuenta
el resultado reemplazado del mismo job.

El protocolo privado v2 usa escalares decimales canónicos para preservar bits sin
cambiar el codec JSON global. V1 conserva encabezado y respuesta independientes.
Los nombres Ocr* evitan colisiones de schemas y mantienen Eq de localizadores.
Los 50 pares de contratos ajenos a index/search son idénticos al baseline; esas
dos operaciones sólo amplían los tipos nombrados de evidencia OCR. API v1/53,
MCP 25/45/46, siete exclusiones y batch 9. Una lectura no concede permiso de run.

## Proyección e interfaz

El índice proyecta únicamente texto reconocido por región, sin metadata ni
umbral de confianza. El hash incluye representación, fuente y proyección. Las
codificaciones UTF-8/PDF/XLSX previas se conservan. La generación válida que
excede 512 fragmentos de proyección se puede leer, pero su selección se rechaza
íntegra: no cambia revisión, selección anterior ni resultados anteriores.

El lector UI mapea explícitamente cuatro operaciones; OCR no cae en XLSX. Muestra
veinte regiones, manifiesto, fuente y límites, con bbox/confianza como cadenas.
Preparación, lectura y búsqueda siguen siendo acciones separadas. Search no
infiere, importa, selecciona ni repara; sus ámbitos preservan páginas y errores.
Los textos inferidos se escapan y explican ausencia/errores posibles del OCR.
Lifetimes y cupos físicos conservan las llamadas iniciadas hasta su terminación.

La aceptación reprodujo un fallo de auditoría del lector con una lectura todavía
pendiente. La auditoría ahora espera todas las entradas y no filtra fallos de
snapshot. Tras dos fallos intermitentes del recorrido agrupado se reforzaron sus
esperas por la apertura identificada y las dos respuestas reales de validación.
Los recorridos normal y con demora forzada pasan; esa evidencia no demuestra la
causa de los fallos originales. Ningún ajuste modifica App, guardias o validación
Rust. Logs, diagnósticos y límites de atribución permanecen en la verificación.

## Alcance y límites

Comparación contra tar y SHA-256, no HEAD: los 488 archivos originales siguen
presentes. Los 217 archivos protegidos de Home, App, parsers/protocolos PDF/XLSX,
migraciones anteriores y gates conservan sus hashes. El lock sólo agrega la
dependencia local del núcleo al protocolo OCR. No se instalaron modelos ni se
activaron proveedores, OAuth, pagos, publicación o pantalla.

Las dos funciones de benchmark y los scripts/corpus correspondientes conservan
sus bytes. Los helpers PDF/XLSX release también son idénticos al baseline; sus
debug tienen otros hashes de build, con fuentes idénticas y ambas copias
conservadas. No se atribuye identidad binaria a partir de identidad de código.

El host macOS compila no-bundle con los tres helpers mediante overlay. El build
no prueba distribución firmada, IPC/foco/cierre nativo o disponibilidad en otros
sistemas. RSS nativo es vigilancia observada, no límite duro RSS/GPU. El corpus
humano acotado y su CER/WER no garantizan transcripción perfecta. OCR de PDF,
ampliaciones PDF/Excel y el cierre completo de 5a quedan fuera de este cambio.
Home permanece activo 9/17; no se sincroniza ni archiva.

## Aceptación y decisión de cierre

Se contrastaron los 19 requisitos/56 escenarios del delta, incluidos veinte
escenarios heredados que conservan su limitación histórica documentada. La
regresión fresca aprobó 319 Rust y catorce etapas backend por perfil, SDK 23 por
perfil, tres rondas raw por perfil y 3442 checks headless de producto. Los cuatro
gates posteriores aprobaron query p95<=300 ms y lock p95<=150 ms; máximo observado
254,539333 ms y 18,811541 ms, respectivamente. No quedan escenarios nuevos sin
contraste o defectos materiales demostrados del recorte; no equivale a una
revisión externa ni aceptación de toda 5a. Sólo los cinco deltas de este cambio
pueden sincronizarse/archivarse. Home queda activo 9/17.

Se sincronizaron sólo estos cinco deltas: 14 requisitos nuevos/5 modificados, con
152 bloques ajenos y 20 nombres heredados preservados. Main tiene 171 requisitos
en 13 capabilities. Archivo 2026-10-17-add-local-image-ocr comprobado con los nueve
hashes intactos; Home permanece activo 9/17. Validación estricta de main aprobada
después de compactar únicamente la frase nueva que había excedido 500 caracteres;
el primer fallo documental queda conservado.
