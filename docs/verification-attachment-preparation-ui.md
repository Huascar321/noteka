# Verificación de preparación explícita

Cambio `add-local-attachment-preparation-ui`, aceptación técnica aprobada;
delta sincronizado/contrastado y cambio archivado el 17/10. No completa la fase 5a. Home permanece 9/17 independiente.

## Baseline

Baseline anterior a editar código: **424 archivos** copiados en tar.gz y
contrastados por SHA-256, ocho binarios CLI/MCP/helpers debug/release copiados y
catálogos completos v12/API v1/52 capturados. Evidencia durable fuera de Git,
directorio señalado por
`~/.local/share/noteka-verification/active-attachment-preparation-path.txt`.
Manifiestos: `baseline-pins.json`, `baseline-review.json`,
`baseline-binary-pins.json`, `catalog-before-debug.json` y release.
Es un baseline v12 de continuación, no recuperación del v11 histórico perdido.
La primera consulta usó por error `catalog`; el comando real es `operations describe`
y las capturas válidas se comprobaron con éxito/52 entradas.

## Controlador y compilación

**19/19 tests** de `tests/attachment-preparation.mjs` aprobados: pasos exactos/CAS,
errores e incertidumbre, páginas, compatibilidad/cero texto, respuestas tardías,
desmontar/remount y espera de mutación cancelada sin iniciarla. Check Svelte
aprobó sin errores/advertencias. El primer check detectó un documento `unknown`;
la integración usa solamente documento semántico reconocido del buffer y conserva
el ingreso manual para formatos futuros, sin hacer cast de contenido desconocido.

## Headless

Primera corrida no inició navegador: faltaba cargo en PATH. Se conservó
`logs/headless-first.log`; se reutilizó Rust 1.99.0 durable existente, sin instalar
ni cambiar configuración global. Los resultados del recorrido real se registran
después de ejecutarlo. No equivale a IPC o aceptación nativa.

El recorrido inicial completo aprobó **150 checks** (75 por tema), seis originales
nuevos, dedupe, UTF-8/PDF/XLSX/cero texto/fallo NUL, tres clases de CAS, 22 trabajos
paginados, transporte perdido después de commit, respuesta tardía/remount y
cancelación/fallo de guardado con buffer. Las consultas son CLI reales; snapshots
antes/después de cada lectura verifican DB completa, y cada operación derivada
comprueba entidades/documentos/revisiones/FTS/historial y hashes de originales.
No se descargó base64. Capturas headless 1280/900 revisadas en ambos temas; sin
overflow horizontal. Evidencia en `runtime/noteka-headless-xofN3y/`.

Fallos previos conservados: timeout de Home en una corrida (causa no demostrada),
selector Notes ambiguo del nuevo test (acotado al nav), esperado `paused` cuando
el núcleo devuelve `queue_paused`, y esperado de éxito para NUL cuando run devuelve
error `nul_in_text` dejando job failed. Se corrigieron tests para contrastar el
contrato real, sin alterar producción ni retirar asserts. Una ejecución final
agrega además CAS de índice separado con generación intacta.

La regresión de Search aprobó **1.248 checks** sin modificar sus fuentes/asserts;
cinco tests de puente aprobaron. El warning conocido de Vite por chunk >500 kB
permanece visible, sin aumentar/ocultar el umbral. El hook `extraction:prepare`
compiló/copió ambos helpers debug offline para aarch64-apple-darwin; no se abrió
Tauri ni se publicó un bundle.

La aceptación headless definitiva aprobó **154 checks** (77 por tema), con CAS del
índice independiente de una regeneración. `logs/headless-final.log` y
`runtime/noteka-headless-FLxvhc/` conservan reports/capturas/snapshots y registro de
operaciones. Las dos respuestas HTTP 500 en consola son la pérdida de transporte
inyectada después de cancel real (una por tema), no excepciones JS no investigadas.
El recorrido anterior completo aprobó **606 checks** (592 + 14 de teclado) en seis
sesiones sin cambiar sus fuentes/asserts; total UI vigente **2.008 checks**.
La guía/matriz distinguen estos resultados del IPC y cierre nativos pendientes.

Host no-bundle release offline/locked con red denegada aprobado, sin ejecutar la
aplicación: `logs/final-host-build.log`. También Clippy all-targets con feature
native-smoke y deny warnings, `logs/final-host-smoke-clippy.log`. Los helpers se
preparan en perfil/target correctos por hook genérico; el script PDF anterior y
Rust/IPC permanecen intactos. No demuestra distribución, firma ni IPC nativo.

Frontend fresco: 13 editor/colas/tema, 14 búsqueda, 19 preparación, cuatro lotes y
cinco puente aprobados. Catálogo completo debug/release idéntico al baseline al
quitar únicamente request_id; API/DTOs/capabilities/whitelist comparados completos.

## Revalidación conjunta con lector

Tras integrar el lector independiente se repitieron **154 + 1.248 + 606 checks**
sobre App/puente actuales; el lector añadió **870** de producto, sin sumar los 848
previos aislados como integración. Siete tests de puente y host no-bundle offline
aprobaron. Logs `headless-joint-*`, `bridge-joint.log`, y reporte `joint-ui-summary.json`.
Se corrigió también el decoder UTF-8 continuo del puente previo; no hay cambios
Rust/contratos/corpus/gates. Pins anteriores conservados, 207 fuentes intactas y
App/puente identificados/revalidados; 217 fuentes actuales congeladas. Los archivos
sin importar de búsqueda conjunta no se presentan como parte de este cierre.

## Aceptación técnica final conjunta

La [aceptación conjunta](verification-local-preparation-closure.md) registra las
36 etapas frescas, cuatro gates nuevos, 222 pins y **3.116 checks headless** sobre
la integración final. Sustituye los conteos parciales de los hitos anteriores,
que se conservan como historia. En el estado final hay 206/209 fuentes originales
iguales; App, puente y ayuda Search son las tres diferencias identificadas y
revalidadas. No se cuentan la regresión/gates compartidos como ejecuciones
independientes para cada cambio. Los tres deltas se sincronizaron/contrastaron y los cambios se archivaron el
17/10; no se declara toda 5a ni aceptación nativa.
