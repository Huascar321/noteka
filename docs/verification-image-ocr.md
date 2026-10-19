# Verificación de OCR integrado — aceptación técnica aprobada

`add-local-image-ocr` cerró 12/12: cinco deltas sincronizados/contrastados y
[archivado](../openspec/changes/archive/2026-10-17-add-local-image-ocr/tasks.md).
Home permanece 9/17. Esta página no
certifica toda 5a ni fases posteriores. [Contrato y límites](image-ocr.md),
[ADR 0020](decisions/0020-local-image-ocr.md) y
[matriz](verification-image-ocr-scenarios.md).

## Evidencia durable

El directorio externo se obtiene desde
`~/.local/share/noteka-verification/active-local-image-ocr-path.txt`.
Baseline previo:488 archivos/catorce binarios y catálogos v12 completos. La copia
v12 inicial, creada antes de editar DDL, conserva tar/prueba de backup/restore.
No se reseteó, limpió, descartó o publicó el repositorio sin commit. Bytecode de
una importación de pruebas se trasladó a evidencia externa; no era parte del
baseline. Los originales de fixtures están fuera de Git, salvo el PNG blanco
sintético MIT de prueba y sus generadores.

## Resultados comprobados

- Base completa actual:319 tests Rust por perfil, diez ignores instrumentales
  declarados; fmt, Clippy con warnings como errores y builds debug/release.
 76 cliente/editor, nueve puente/lotes más dos del puente de lector, todos pasan.
 La frontera de proyección comprueba 512/513 fragmentos sin selección parcial;
 la publicación OCR real en la frontera 32 MiB con UTF-8/PDF/XLSX conserva los
 resultados, permite reemplazar sólo su propio job y rechaza exceso sin purga.
- Protocolo:53 casos por combinación wire 1/wire 2 × debug/release; corpus humano
  sin retocar. CER5/706 (0,7082%) y WER5/106 (4,7170%) en el subconjunto limpio
  declarado; no medida de exactitud universal. Prueba decimal de >20.000 valores,
  incluidos subnormales/cero negativo, conserva bits.
- Integración CLI real,22 casos por perfil con Vision local y red denegada.
  Oráculo Python independiente de representación desde la salida nativa exacta
  capturada para esa publicación; no usa un segundo reconocimiento como golden.
  Lectores/selección/búsqueda no corren helper. Backup/restore de OCR real sin
  helper y corrupción interna con hash externo de SQLite recalculado: rechazo
  sin destino parcial, generación y originales anteriores conservados.
- Migración real v12→13 en ambos perfiles:37→39 tablas. Filas, DDL previo,
  originales, cola/control/ownership, desconocidos y hashes/cursores de índice
  exactos; recibo previo invalida por versión. Rollback antes/después de versión,
  rechazo de cliente v12 y futuro14 antes de escrituras, backup restaurado por
  cliente anterior en otra ruta. Ambos runtimes de verificación sin helpers.
- Los 50 pares input/output ajenos a index/search permanecen exactos frente a
  los 52 anteriores. Sólo las adiciones OCR nombradas amplían index/search; batch 9,
  error schema y cuotas exactos. API v1/53, MCP 25/45/46; SDK real23/23 en ambos
  perfiles y tres rondas raw por perfil aprobados.
- Host macOS no-bundle compiló offline/red denegada con overlay y los tres
  helpers preparados. Se usó target externo para separar artefactos de pruebas.
  No se lanzó una ventana; no acredita IPC/foco/cierre nativo de Home.
- Tras la suite completa se detectó que tauri-build había copiado el staging
  debug compartido a ambos directorios de perfil. Se conservaron las seis copias
  y sus hashes, idénticos al staging. Se recuperaron solamente esos artefactos
  generados desde los builds reales ya fijados; no se tocaron fuentes/originales
  ni se presentó esa copia como reconstrucción reproducible. Se repitieron los
  catorce tests OCR por perfil con sus helpers reales fijados y se comprobaron
  después los dieciséis hashes. Una reproducción controlada mostró que Cargo
  reemplaza el sidecar staged por su build release; el hash de paquetes PDF/XLSX
  concretos difiere del workspace y ambas variantes se conservan. El orden de
  [verificación por perfil](development.md) queda documentado.
- Gate OCR adicional:9 generaciones/512 regiones cada una,8 MiB de texto,
 5 warmups/20 muestras por consulta. P95 CLI release **172,512208 ms** con hits y
  **160,37025 ms** sin hits (<=300), incluyendo arranque/sandbox y validación
  completa. Derivados sintéticos explícitos: no medición de inferencia/calidad.
  Receta/hash fijados antes de medir; no cambia los cuatro gates heredados.

## Regresión y gates finales

Ambos perfiles completaron sus catorce etapas, con oráculos/procesos PDF/XLSX,
revalidación, veinte rondas de adjuntos, tres rondas raw, SDK 23 y CLI OCR 22.
El caso XLSX de 10.000 celdas se recorrió de una celda por página, con validación
íntegra y sin reducir corpus/cuotas. La corrección del conteo de llamadas CLI se
verificó con 128 llamadas/21 ejecuciones nativas por perfil, sin cambiar casos o
aserciones del oráculo.

Los **3442 checks headless de producto** pasaron: 606 heredados,1248 evidencia,
154 preparación,870 lector,238 agrupada y 326 OCR. Los diagnósticos/repeticiones
no se suman al producto; ambos temas y cero requests externos. El último recorrido
OCR también comprobó los hashes de los dieciséis artefactos fijados. No son pruebas
de IPC/cierre/foco nativo.

Cuatro gates release consecutivos después de terminar la regresión, con binarios,
corpus, scripts y funciones de benchmark fijados. Query p95<=300 ms y lock
p95<=150 ms en todos los casos, sin cambiar validación/cuotas/objetivos. Rangos
entre consultas de cada gate; no promedios globales.

| Gate | Query p95 (ms) | Máximo lock p95 (ms) |
| --- | --- | --- |
| mixed-1 | 165.927000–254.539333 | 14.217500 |
| mixed-2 | 166.670417–210.647834 | 14.605125 |
| unchanged-1 | 109.808625–128.798959 | 18.189000 |
| unchanged-2 | 117.484667–135.567625 | 18.811541 |

Los cuatro gates aprobaron su primera cadena actual. Logs y muestras en
`final-gates-stages.log`, `logs/final-gate-*` y `acceptance-summary.json`;
`headless-acceptance.json`, `rust-acceptance.json` y los manifiestos de código y
artefactos registran los insumos. La comprobación de conservación incluye 217
archivos protegidos y las dos funciones de benchmark, idénticos al baseline.
El gate OCR adicional anterior mide otro corpus y no sustituye estos cuatro.
No garantiza estos p95 para cualquier corpus, caché, hardware o carga.

Los gates fallidos de cierres anteriores permanecen en sus verificaciones, con
causa no demostrada; no se vuelven aprobados o históricos nuevos por este cierre.

## Fallos y revisión conservados

- Primer acceso al v12 real falló porque canonical consultaba la tabla OCR antes
  de013. No escribió el workspace; se agregó comprobación de versión efectiva,
  regresión SQL específica y migraciones independientes actuales aprobadas.
- El caso privado v2 sin encabezado no puede negociar versión. El helper conserva
  su error v1 por defecto; el consumidor v2 lo rechaza. El probe inicial pretendía
  validarlo como v2; se corrigió ese supuesto, sin aceptar éxitos cruzados.
- La revisión exigió volver a comprobar formato/metadata dentro del writer lock y
  comparar ownership de chunks dentro de SQL. Se agregaron guardias y prueba
  nativa de cambio de metadata tras extracción, hooks alcanzados y rollback de
  publicación. La cadena previa se detuvo y sus logs quedaron aparte antes de
  fijar la aceptación actual; no fue un fallo de gate ni una excepción.
- El recorrido lector falló al auditar una lectura pendiente. El log original no
  guardaba ese estado; se agregó trazabilidad y una reproducción con 500 ms en
  relation.list mostró IN_FLIGHT, sin snapshot de escritura. La espera acotada
  exige que todas las llamadas terminen y conserva la aserción de cero writes:
  no omite entradas falsas. Reproducción completa 870 checks y recorrido normal
 870 aprobados. No se cambió el núcleo por ese fallo del harness.
- El recorrido agrupado falló en la comprobación UTF-8; un diagnóstico posterior
  pasó y otro falló antes, al verificar el buffer. Los logs originales no permiten
  demostrar la causa de esos fallos. El harness ahora espera la nota identificada
  tras openOriginal y las dos respuestas reales de validación antes de comprobar
  sus errores en el DOM, sin retirar asserts ni cambiar el producto. Recorrido
  normal 238 y diagnóstico con 300 ms en entity.get 238 aprobados; no se afirma una
  causa del núcleo o una optimización a partir de esa evidencia.
- Conteos antiguos52/24 y fixtures SQL que retenían tablas13 al simular épocas
  previas se actualizaron; fingerprints históricos no se modificaron. Se guardan
  también los errores de compilación de las nuevas pruebas antes de corregirlos.

El runner UI heredado compila CLI de manera independiente y produjo una
variante debug distinta del build de workspace. Ambas copias/hash están en
evidencia externa, con catálogos completos iguales y los 50 pares previos exactos;
no se afirma identidad binaria. Los gates de rendimiento usan release.
La comparación exacta con el v11 histórico sigue sin ser repetible, según el
[criterio previamente autorizado](xlsx-search-evidence-recovery.md); las fixtures
SQL v10/v11 actuales no son clientes históricos ni recuperan esos archivos.

## Sincronización y archivo comprobados

Los cinco deltas contienen 19 requisitos/56 escenarios: cinco MODIFIED y catorce
ADDED. Main pasa a 171 requisitos en 13 capabilities; 152 bloques no afectados,
veinte nombres de escenarios heredados, títulos/Purpose y ocho capabilities
ajenas se conservan. Extraction 38→48, text-search 23→24, core 26→27,
workspace 14→16 y MCP 13→13. Las nueve piezas del cambio, incluida .openspec.yaml,
conservan exactamente sus hashes tras el archivo 2026-10-17-add-local-image-ocr.

La primera validación estricta de main tras el sync encontró una descripción
modificada de 506 caracteres. Sólo se compactó la frase nueva de continuación
(497), conservando los escenarios y el texto heredado. Se repitió contraste y
validación estricta; el log fallido permanece separado. No fue un fallo de gate,
cambio de código o excepción al límite. El archivo emitió una advertencia no
bloqueante sobre tamaño de propuesta, conservada en su log.

## Límite de la conclusión

Lectura OCR no equivale a transcripción perfecta, tabla reconstruida, geometría
del original binario o descripción visual. Memoria nativa es RSS observado con
muestreo, no allocator Rust ni límite duro RSS/GPU. No se instalaron modelos,
activaron proveedores/OAuth/pagos/publicaciones o usaron ventanas/pantalla nativa.
PDF amplio/Excel y la salida completa de 5a siguen pendientes de recortes propios.
