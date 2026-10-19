# Tasks

Estado: **17/17 tareas implementadas/verificadas**. Evidencia en
`docs/verification-evidence-search-ui.md`. `openspec validate` no prueba la aplicación. Capability única:
[`local-text-search`](specs/local-text-search/spec.md), un requisito modificado,
seis añadidos y diecinueve escenarios (uno heredado). Home sigue 9/17, separado.

## 1. Baseline and synthetic sources

- [x] 1.1 Conservar copias de CLI/MCP/helpers debug/release existentes junto al baseline durable pre-UI, verificar hashes de los 409 archivos y catálogo completo v12 y registrar fuentes congeladas (Rust/host/contratos/locks/corpus/gates/Home/otros archivos de cambios). Verificar copia byte a byte y hash actual contra pins; documentar rutas/procedencia sin afirmar recuperación de v11.
- [x] 1.2 Preparar fixture propia de evidencia en workspace nuevo confinado usando CLI real y generadores sintéticos intactos: UTF-8/PDF/XLSX, Unicode/HTML inerte, más de 20 hits, caché/fórmula/porciones largas, selección vacía, compatible sin seleccionar, intento anterior y job sin generación; inyectar sólo configuración futura incompatible por SQL de prueba. Verificar páginas/localizadores/cobertura con CLI y goldens, y documentar preparación y snapshots; no introducir hits prefabricados. Trazabilidad: «Paridad de consulta y filtros», ambos escenarios de cobertura y tres localizadores.

## 2. Bounded read controller

- [x] 2.1 Implementar controlador tipado de adjuntos con `ReadLifetime` y pool físico compartido `ReadQueue(1)` sin cambiar helpers/Home; añadir tests Node y comando de ejecución, y documentar vida lógica frente a IPC no cancelable. Verificar promesas controladas de éxito/rechazo/finally, cambio de criterios, dispose/remontaje, cancelación de espera y máximo un IPC activo, sin estados tardíos. Trazabilidad: «Respuesta tardía de otro workspace o ámbito» y «Editar criterios invalida página anterior».
- [x] 2.2 Implementar primera/siguiente/reinicio con límite 20, una página reemplazable, cursor opaco y criterios enviados; invalidar al editar y conservar errores del núcleo sin truncar input, reintento automático ni página anterior vigente. Añadir tests de filtros omitidos/combinados, consulta exacta multibyte, orden y sustitución de páginas, cero hits frente a estado inicial, conflicto/storage/busy/transporte, y documentar reglas en la guía de Search. Trazabilidad: consulta/validación/estados y tres escenarios de paginación.

## 3. Evidence presentation and Search integration

- [x] 3.1 Crear panel de adjuntos con texto Svelte escapado y detalle completo de fuente/IDs/hashes/generación/extractor/ordinal/flag anterior, sin HTML interpretado, linkificación ni operaciones N+1; documentar que inspeccionar no revalida ni abre el archivo. Verificar en headless texto canónico Unicode/espacios/saltos sin slicing y contenido activo inerte sin requests remotos. Trazabilidad: «Texto Unicode y procedencia completa», «Contenido activo es sólo texto» y «Consultar adjuntos no es procesarlos».
- [x] 3.2 Presentar los tres localizadores, los seis campos XLSX, límites PDF, fórmula/caché/rangos parciales y cobertura/perfil/versión/revisión devueltos; documentar fuentes y categorías sin inferir dueño/nombre/visibilidad de hoja o cobertura universal. Verificar con fixture real contra CLI/lector/golden, incluyendo hojas ocultas, cero hits/selecciones vacías/incompatibles, scopes de verificación y distinción ordinal de fragmento/celda. Trazabilidad: localizadores y cobertura completos.
- [x] 3.3 Integrar selector de ámbitos en App preservando búsqueda/apertura/rebuild de notas y editor/guardias; impedir `search.query` en adjuntos, invalidar lectura de notas al alternar y montar panel por workspace/ámbito/sección. Conectar «Actualizar» sin doble consulta, documentar uso y preparación explícita por CLI/MCP. Verificar notas heredadas, filtro exclusivo por ámbito, buffer intacto al consultar/alternar y guardar/descartar/cancelar al navegar. Trazabilidad: los tres escenarios de «Búsqueda de escritorio offline» y «Buffer y guardia preservados».
- [x] 3.4 Añadir estilos sólo del panel, controles nombrados, estados accesibles y foco visible, sin cambiar shell/Home/nav/clima/secciones. Verificar Enter/Tab/Space, texto largo y detalle/hash completo en claro/oscuro al ancho de lista actual, y comparar archivos/selectores Home con baseline. Registrar límites de UX y que headless no acredita IPC/cierre/foco nativos. Trazabilidad: «Teclado y temas sin cambio de Home».

## 4. Headless integration and preservation

- [x] 4.1 Extender sólo puente/runner headless con preparación acotada de fixture, registro de operaciones y snapshot lógico/fuentes/generaciones/índice; añadir tests de origen/rutas/symlinks y verificación de inmutabilidad por lectura de producto. Verificar que montar/buscar/inspeccionar no llama extracción/indexación/read/base64 y que siguen confinamiento, red CLI denegada, loopback y perfil efímero. Documentar qué se inyectó/preparó por prueba y qué se ejecutó por UI real.
- [x] 4.2 Añadir recorrido headless separado con paridad CLI de los tres extractores, filtros AND, paginación >20 sin acumulación/duplicación, límites inválidos multibyte/UUID, fuente alterada y cursor vencido por acción CLI deliberada. Verificar página/cobertura descartadas ante error, ausencia de reparación/reinicio y snapshots de sólo lectura; registrar checks y razones. Trazabilidad: consulta, localizadores, cobertura y paginación completos.
- [x] 4.3 Retener respuestas reales ya obtenidas (éxito y rechazo) y cambiar texto, filtro, ámbito, sección y workspace antes de liberarlas; añadir controles de prueba sólo donde haga falta. Verificar que no reaparecen hits/cursor/cobertura/error/loading obsoletos, que nuevas consultas usan el workspace correcto y que el pool no excede un IPC; registrar intercalados. Trazabilidad: aislamiento y edición de criterios.
- [x] 4.4 Ejecutar seis sesiones headless previas más los recorridos nuevos en ambos temas, reapertura/restauración y teclado/guardias; conservar los 606 asserts anteriores sin retirarlos o convertirlos en éxito simulado. Verificar cero intentos HTTP externos y Home intacto 9/17; registrar conteos antiguos/nuevos y límites del puente por separado, sin iniciar pruebas nativas.

## 5. Integrated verification and readiness

- [x] 5.1 Ejecutar contratos/check/build y tests frontend nuevos/existentes, workspace/host fmt/check/Clippy y tests Rust debug/release offline/locked; verificar MCP y oracles existentes relevantes a UTF-8/PDF/XLSX y build Tauri no-bundle sin ventana. Guardar logs cerrados y comparar catálogo/DTOs/whitelist completos contra pre-UI v12: ninguna operación, contrato, dependencia, migración o fuente congelada puede cambiar inadvertidamente. Esto es regresión fresca, no recuperación de evidencia v11.
- [x] 5.2 Correr dos gates release consecutivos UTF-8/PDF y dos mixtos XLSX con corpus/pins/gates intactos, query p95 <=300 ms y lock p95 <=150 ms; guardar muestras/percentiles/metodología y fallos. Verificar todos los límites/validaciones existentes sin excepciones y separar latencia headless/render del benchmark Rust; si falla, investigar sin relajar objetivos.
- [x] 5.3 Consolidar guía y evidencia/matriz iniciadas en los grupos previos: mapear los diecinueve escenarios a asserts, resultados y limitaciones reales; actualizar plan/workflow/config/README/cola de revisiones sólo al estado efectivamente implementado. Verificar enlaces y comandos documentados, cobertura de seis requisitos añadidos más el heredado ampliado, y que 5a/Home/aceptación nativa/modelos/remotos no se declaran completos.
- [x] 5.4 Revisar diff final contra baseline pre-UI y alcance de propuesta/diseño/delta; verificar hashes y ausencia de cambios en Home/otros cambios activos/archivos archivados, Rust/contratos/locks/parsers/corpus/gates. Registrar revisión técnica, resolver hallazgos y comprobar que no hay datos personales, credenciales ni runtime en Git; una eventual decisión arquitectónica nueva requiere actualización coherente y ADR antes del cierre.
- [x] 5.5 Validar estrictamente el cambio y todas las specs/cambios, comprobar 17 tareas con evidencia y preparar fuera de main un merge de sólo este delta; verificar que preserva el escenario heredado de notas y todos los requisitos/escenarios ajenos, sin tocar deltas de Home ni resincronizar archivos previos. El resultado de validación es documental; la preparación para cerrar exige además las pruebas/revisión anteriores.

## Workflow follow-up

- Cuando las 17 tareas estén verificadas, sincronizar sólo el delta implementado
  de `local-text-search`, contrastar el merge exacto y ejecutar validación estricta
  de las specs resultantes; no publicar comportamiento meramente propuesto.
- Archivar únicamente `add-local-evidence-search-ui`, verificar ubicación, enlaces,
  manifiestos/evidencia y estado posterior. Home debe seguir activo 9/17 y sin
  sincronización/archivo. Si el merge ya se aplicó y contrastó manualmente, usar
  `--skip-specs` sólo para evitar aplicarlo por segunda vez, no para omitirlo.
- Estas acciones posteriores al trabajo verificado no cuentan como implementación
  realizada por la mera existencia de documentos ni como tareas previas al archivo.

## Closure

El 16/10/2026 se sincronizó únicamente `local-text-search`, se contrastó byte a
byte con el merge staged y se validó en modo estricto. Archivado como
`2026-10-16-add-local-evidence-search-ui`; diecisiete tareas verificadas y Home
activo 9/17 intacto. Ver [evidencia y limitaciones](../../../../docs/verification-evidence-search-ui.md).
