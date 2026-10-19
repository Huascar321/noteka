# Noteka: plan de producto, arquitectura y desarrollo

Fecha de referencia: **11 de octubre de 2026**, zona `America/La_Paz`.

Este documento está escrito para una persona o IA que no participó de las conversaciones originales. Contiene el objetivo del producto, las decisiones tomadas, una arquitectura propuesta, requisitos, fases y criterios para continuar. Los chats son antecedentes; no hacen falta para ejecutar el plan.

**Estado real al 17 de octubre:** fases 0–4, búsqueda textual, cola/UTF-8, [PDF textual acotado](verification-pdf-extraction.md), [búsqueda de adjuntos](attachment-search.md), MCP local, [extracción XLSX acotada](xlsx-extraction.md) y [búsqueda XLSX](xlsx-search.md) verificados y archivados. SQLite v13/API v1/53 tras la integración OCR de imágenes verificada; nueve mutaciones batchables y documentos v1/v2/v3 conservados, sin modelos ni proveedores. Comparación histórica exacta v11 no repetible tras perder el baseline, según [criterio alternativo autorizado](xlsx-search-evidence-recovery.md). **Home activo 9/17, sin archivar**; [verificación](verification-home.md): los 606 checks headless no acreditan aceptación nativa. [P95 búsqueda de notas histórica](search.md) 22,92 ms/10.000 notas. PDF amplio, OCR de PDF, ampliaciones Excel, embeddings/IA y resto de 5a pendientes. Desarrollo local autónomo autorizado; [revisiones](review-queue.md). UI headless: no pantalla sin autorización específica.

**Cierre previo MCP:** [`add-local-mcp`](../openspec/changes/archive/2026-10-15-add-local-mcp/proposal.md): **22/22 tareas verificadas**, quince requisitos/45 escenarios (cinco heredados); dos deltas sincronizados y cambio archivado. SQLite v10/API v1/51 y [evidencia](verification-mcp.md): **235 tests Rust por perfil**, 17 SDK por perfil, tres rondas raw por perfil, corpus de 235 casos parser/CLI PDF por perfil y 606 checks headless. Los cierres previos de adjuntos v10/51 y PDF v9/49 conservan su evidencia independiente.

**Home activo:** [`add-local-home-dashboard`](../openspec/changes/add-local-home-dashboard/proposal.md), código implementado y **9/17 tareas completamente verificadas**; [revisión final cerrada](review-home.md), ocho tareas con aceptación nativa pendientes, no ausencia de código. [Home](home.md) tiene quince vistas de datos originales, referencias/resúmenes, Today por due_date <= día civil y temas claro/oscuro, sin alterar workflows previos ni migrar. Goals/clima/fotos/auxiliares siguen pendientes; [matriz](../openspec/changes/add-local-home-dashboard/design.md). Recorrido headless aprobado: **592 checks compartidos + 14 de teclado/intercalados**, sin ocupar pantalla. No se reemplaza evidencia Tauri por evidencia del puente CLI; falta aceptación nativa, sin archivo prematuro.

**Lector independiente cerrado:** [`add-bounded-xlsx-reader`](../openspec/changes/archive/2026-10-15-add-bounded-xlsx-reader/proposal.md), **8/8 tareas**, seis requisitos/dieciséis escenarios sincronizados y contrastados con main, archivado. [Lector aislado](xlsx-reader.md), entregado sin activar cola/SQLite/API/MCP/FTS/UI en su baseline histórico v10/51. Números léxicos y fórmulas/caches separados, ZIP/XML cerrado y contención compartida con PDF; [ADR 0016](decisions/0016-bounded-xlsx-reader.md). [Aceptación final3 histórica](verification-xlsx-reader.md): 216 casos por parser/helper y 245 tests Rust por perfil, regresión PDF/MCP y 606 checks headless. Query p95 121,38–146,34 ms en dos gates intactos. La integración siguiente se verificó separadamente; no modifica Home ni satisface toda 5a/5b.

**Extracción XLSX archivada:** [`add-local-xlsx-extraction`](../openspec/changes/archive/2026-10-15-add-local-xlsx-extraction/proposal.md), **11/11 tareas**.
Cola explícita, persistencia canónica recuperable, migración aditiva v11 y lectura
tipada API v1/52/CLI/MCP 24/44/45; [ADR 0017](decisions/0017-persisted-xlsx-cells.md).
[Aceptación final3](verification-xlsx-extraction.md): 261 tests Rust por perfil,
216 casos CLI/parser/helper XLSX por perfil, regresión PDF/MCP y 606 checks headless.
Dos gates FTS intactos query p95 111,15–146,42 ms frente a 300 ms; locks y RSS
medidos. Cuatro deltas/14 requisitos/44 escenarios contrastados con main.
Sin indexación XLSX, UI/Home, modelos o remotos; no completa toda 5a/5b.

**Búsqueda XLSX archivada:** [`add-local-xlsx-search`](../openspec/changes/archive/2026-10-16-add-local-xlsx-search/proposal.md), **19/19 tareas**, cinco deltas sincronizados y contrastados (14 requisitos/49 escenarios).
Revalidación técnica durable aprobada el 16/10: código v12/API v1/52, proyección
explícita de campos persistidos; 287 Rust por perfil, corpus/procesos/MCP y 606
checks headless aprobados. Dos gates mixtos **167,02–270,74 ms** y dos intactos
**114,31–152,19 ms** p95, con target 300 ms, lock <=150 ms y caps/validación intactos.
[Guía](xlsx-search.md), [evidencia](verification-xlsx-search.md) y [limitación
autorizada](xlsx-search-evidence-recovery.md): se perdieron los baselines/logs
originales y **no se repitió** la comparación exacta de archivos/50 contratos o
cliente v11. Las fixtures SQL no se presentan como independientes. Los 32 escenarios
heredados y 75 requisitos no afectados se conservaron; Home 9/17 sigue aparte,
sin pantalla/modelos/remotos.

**Recorte previo cerrado de 5a:** [`add-local-evidence-search-ui`](../openspec/changes/archive/2026-10-16-add-local-evidence-search-ui/proposal.md),
**17/17 tareas implementadas/verificadas**. Expone los fragmentos
UTF-8/PDF/XLSX ya seleccionados, en ámbito separado de notas/entidades, con
procedencia/cobertura y paginación acotada. La propuesta no añade operaciones ni
migraciones ni ejecuta extracción/indexación desde la búsqueda. Baseline, fixture
y controlador verificados; UI aprobó 1.248 checks nuevos y los 606 heredados,
regresión Rust/MCP, cuatro gates y revisión documental aprobados (primer fallo
conservado); único delta sincronizado/contrastado y cambio archivado.
[Evidencia](verification-evidence-search-ui.md). Query p95 final 121,37–235,47 ms,
lock máximo 18,94 ms; sin cambios de validación/caps/targets o núcleo Rust.
Home permanece independiente 9/17.

**Tres recortes nuevos de 5a verificados el 17/10:**
[preparación explícita](attachment-preparation-ui.md),
[lectura completa por páginas](attachment-reader-ui.md) y
[consulta agrupada](grouped-search.md), con 9/9, 6/6 y 6/6 tareas verificadas,
respectivamente. [Aceptación conjunta](verification-local-preparation-closure.md):
3.116 checks headless, 287 Rust por perfil y cuatro gates frescos aprobados;
catálogo/schema/MCP/cuotas intactos. Los tres deltas están sincronizados/contrastados y los cambios archivados por separado.
OCR y ampliaciones PDF/Excel siguen pendientes; **5a todavía no está completa**.
Home permanece activo 9/17.

**Lector OCR independiente verificado el 17/10:**
[guía](ocr-reader.md) y [aceptación](verification-ocr-reader.md).
PNG/JPEG mediante Vision local r3/fast, sin instalaciones, con protocolo Rust y
53 casos por perfil. En el corpus limpio declarado: CER 5/706 (0,7082%), WER
5/106 (4,7170%); no es garantía general. No hay integración con cola/SQLite/API/
FTS/UI: ésa sigue como próximo recorte de 5a. Lector cerrado 6/6, delta sincronizado/contrastado y archivado el 17/10.
Home permanece activo 9/17; no se instalan pesos ni se activan proveedores remotos.

**Punto de parada vigente, ampliado el 17/10:** continuar por las dependencias de
5a → 5b → 5c hasta la **fase 6 inclusive** (6a proveedores, 6b Drive/Docs/Sheets y
6c MCP según alcance documentado), con informe acumulado al cerrar o identificar
una intervención imprescindible. No iniciar fases 7/8. La ampliación no autoriza
instalar pesos/modelos, activar proveedores/pagos, OAuth, reutilizar sesiones
internas, publicar ni usar pantalla. Se implementa y verifica lo independiente;
acceso/inferencia/OAuth reales requieren la intervención explícita correspondiente,
sin presentar mocks como aceptación real. Home conserva su brecha nativa.

**Fase 4 cerrada:** [`extend-agent-operations`](../openspec/changes/archive/2026-10-13-extend-agent-operations/proposal.md), 22/22: preview aislado/nueve mutaciones/apply atómico y suscripción CLI. Su evidencia histórica usa 40 operaciones/v7 sin migración; [contrato actual](batches.md), [ADR 0010](decisions/0010-atomic-local-change-plans.md) y [evidencia](verification-phase-4.md). Extracción evoluciona schema/huella sin ampliar whitelist. Independiente de Home; no habilita modelos/MCP/remotos.

**Extracción literal cerrada:** [`add-local-extraction-queue`](../openspec/changes/archive/2026-10-13-add-local-extraction-queue/proposal.md), 20/20 tareas, tres deltas sincronizados y archivado. Cola pausada por defecto, worker explícito y UTF-8 literal `.txt`/`.md` administrado <=1 MiB; baseline SQLite v8/48 operaciones. [Guía](extraction.md), [evidencia histórica](verification-extraction.md), [ADR 0011](decisions/0011-explicit-local-extraction.md). PDF acotado se entrega por separado; PDF amplio/Excel/OCR/indexación y resto de 5a pendientes. Home no se sincroniza ni archiva.

**Recorte PDF cerrado:** [`add-local-pdf-text-extraction`](../openspec/changes/archive/2026-10-14-add-local-pdf-text-extraction/proposal.md), **21/21**, sincronizado y archivado. Type1/Standard/WinAnsi acotado, helper descartable, páginas/localizadores derivados, lector público separado y migración aditiva v9; [guía](pdf-extraction.md), [evidencia](verification-pdf-extraction.md), [investigación/licencias](../openspec/changes/archive/2026-10-14-add-local-pdf-text-extraction/research.md). Los 48 contratos previos y UTF-8 literal se conservan. No PDF genérico, OCR, Excel, indexación, modelos o remotos; no cambia Home/aceptación nativa ni completa 5a/5b.

El corpus propio MIT tiene 235 casos con golden independiente, framing histórico/
linearización y precedencia xref/free/hybrid. Se prueba por parser y CLI reales,
no sólo por un example. Límites/memoria/fallos y mediciones locales se registran
por separado de las garantías del formato y de distribución.

**Recorte cerrado de 5a:** [`add-local-attachment-search`](../openspec/changes/archive/2026-10-14-add-local-attachment-search/proposal.md), **19/19 tareas**. Código v10/51 con FTS explícito independiente, fuentes/localizadores tipados y verificación exacta de copias/postings; [guía](attachment-search.md), [ADR 0013](decisions/0013-explicit-attachment-index.md). [Verificación](verification-attachment-search.md): 212 tests Rust por perfil, migraciones/20 rondas CLI por perfil, 49 contratos anteriores preservados y 606 checks headless. Matriz de 15 requisitos/40 escenarios nuevos y revisión externa completadas; un requisito/cinco escenarios heredados de migración reconciliados con la versión vigente y verificados nuevamente en debug/release. **6.1 aprobada:** dos benchmarks consecutivos con query p95 **214,20–231,07 ms** sobre el corpus fijado de 8 MiB frente a <=300 ms, lock máximo p95 **17,66 ms** frente a <=150 ms; scratch físico observado **21.254.144 bytes**. El usuario eligió optimizar sin cambiar objetivo/caps ni retirar validación; se conservaron los baselines fallidos. No garantiza 300 ms en cualquier corpus/caché. Tres deltas sincronizados, contrastados con main y cambio archivado; sin UI/modelos/remotos ni cambios de Home, búsqueda de notas o lectores. No cierra 5a ni habilita por sí solo 5b.

Optimización de implementación de este recorte: NO_MUTEX sólo en la conexión privada de índice/búsqueda, copias ordenadas exactas y reutilización de tokenización de textos byte a byte iguales sin omitir postings. Hash JSON v1 con vistas prestadas y `sha2/asm` sólo en aarch64; mismo algoritmo/bytes persistentes, versión sha2 conservada y una única dependencia añadida al lock (`sha2-asm 0.6.4`). Son decisiones explícitas en [ADR 0013](decisions/0013-explicit-attachment-index.md), con regresión y mediciones, no activación de IA/proveedores ni relajación del target.

En la validación MCP se optimizó además el recorrido exacto de instancias mediante la API pública FTS5 ya incluida, con fallback íntegro, bitsets/deadlines y tests diferenciales: [ADR 0015](decisions/0015-exact-fts5-instance-traversal.md). No cambia formatos ni el criterio de 300 ms; dos gates con query p95 **134,96–204,81 ms** y regresión fresca aprobados, con fallos anteriores conservados en [verificación MCP](verification-mcp.md).

**Recorte de 6c cerrado, al 15 de octubre:** [`add-local-mcp`](../openspec/changes/archive/2026-10-15-add-local-mcp/proposal.md), **22/22 tareas**, dos deltas sincronizados y cambio archivado; [evidencia/límites](verification-mcp.md) y [guía](mcp.md). Scope Rust/binario stdio con 23/43/44 herramientas, siete exclusiones y catálogo CLI/core v1/51 conservado. Pasan **235 tests Rust por perfil**, 17 SDK por perfil, tres rondas raw por perfil y 606 checks headless; ambas eras y auto probados. Query p95 **134,96–204,81 ms** en dos gates con corpus/caps/validación intactos. No completa todo 6c ni activa modelos, proveedores, red externa o pantalla.

## 1. Qué queremos construir

**Noteka es una aplicación personal de escritorio para organizar conocimiento y productividad, con datos locales y capacidades de IA integradas desde el diseño.** Combina tres componentes:

1. Un espacio de trabajo de notas, tareas, proyectos, áreas, recursos, personas y archivo, inspirado en las capacidades esenciales de Ultimate Brain de Thomas Frank.
2. Un editor de bloques y bases de datos relacionadas, visualmente inspirado en Notion, capaz de alojar pequeños sistemas personalizados con propiedades, fórmulas, rollups y vistas.
3. Un harness de agentes que pueda recuperar conocimiento con evidencia y construir o modificar esos sistemas mediante operaciones estructuradas y verificables.

El usuario quiere aprovechar sus notas y adjuntos para responder preguntas, y también pedirle a una IA que cree estructuras complejas dentro de Noteka. Ejemplo: una página para administrar una suscripción compartida, relacionar participantes con su configuración, calcular importes y mostrar pagos pendientes. Ese sistema debe construirse con las capacidades genéricas del producto, sin programar una pantalla especial para cada caso.

**Local-first** significa que el contenido original se conserva y se opera localmente. **AI-first** significa que los datos tienen estructura, procedencia y operaciones accesibles para agentes; no significa que escribir una nota dependa de un LLM.

El núcleo funcionará sin Internet ni un servidor externo. Los modelos locales requieren disponer previamente de sus pesos y runtime. Google Drive, modelos alojados, OAuth y descargas iniciales sí requieren red. Esas dependencias no deben bloquear la edición, lectura y organización local.

## 2. Decisiones del usuario y límites

| Tema | Decisión adoptada | Consecuencia |
| --- | --- | --- |
| Nombre | Noteka | Usar este identificador en producto y documentación. |
| Uso | Personal, inicialmente en macOS | No comenzar con SaaS, cuentas multiusuario o colaboración. |
| Stack | Tauri 2, Svelte 5, TypeScript, Rust y SQLite | Mantener la lógica de dominio en Rust. |
| Editor | Bloques enriquecidos, Tiptap + ProseMirror | El modelo persistente debe sobrevivir al editor. |
| Referencia funcional | Capacidades esenciales de Ultimate Brain | Priorizar notas, tareas, proyectos, PARA, personas, relaciones y vistas útiles. |
| Referencia visual | Notion, preferentemente oscuro | Sidebar, editor centrado, propiedades y bases incrustadas. |
| LLM | Proveedores y modelos intercambiables | Cambiar modelo no debe exigir cambiar arquitectura o datos. |
| Embeddings | Jina Embeddings v5 Text Small como preferencia inicial | Configuración independiente del LLM y posibilidad de reindexar con otro modelo. |
| Modelo local habitual | Qwen3.5-9B como candidato inicial | Su adopción operativa depende de evaluaciones reales en el harness. |
| Experimento | Qwen3.5-4B, evaluación base y posterior LoRA | Es adicional; no condiciona el producto. |
| Herramientas externas | CLI primero, MCP después | Ambas superficies reutilizan operaciones del núcleo. |
| Google Drive | API oficial, archivos y carpetas asociados a entidades | Navegación y previews propios de Noteka cuando los permisos lo permitan. |
| Google Docs | Previsualización y apertura externa | El archivo original sigue en Google. |
| Google Sheets | **Solo previsualización y botón para abrir en el navegador** | No implementar edición de celdas ni sincronización de escrituras. |
| Google Fotos | **Descartado** | No incluir proveedor, autenticación ni indexación de Fotos. |
| Sincronización entre Macs y web | Posteriores | Mantener posibilidades de evolución sin construirlas ahora. |

Se adopta la funcionalidad esencial de Ultimate Brain, no la promesa de copiar cada módulo de su template. Su documentación muestra tareas, notas, proyectos y tags como núcleo; las áreas y recursos pueden representarse mediante tags tipados. Noteka utilizará esa base y un modelo propio. [Referencia oficial](https://thomasjfrank.com/docs/ultimate-brain/start-using-ultimate-brain-the-simple-way/).

No se promete compatibilidad total con la sintaxis o importación de Notion, ni calidad perfecta de todos los modelos. Libros, recetas, calendarios externos, captura web y automatizaciones avanzadas pueden proponerse como módulos posteriores; no son condiciones ocultas de la primera entrega. **Goals y la composición Home fueron solicitados explícitamente con siete capturas el 12/10/2026**: ver [referencia confirmada](home-reference.md); su alcance visual no convierte funciones pendientes en implementadas.

## 3. Qué está confirmado y qué es propuesta

Las decisiones de la sección 2 forman el alcance adoptado. La estructura de tablas, nombres de operaciones, políticas de concurrencia y orden fino de implementación de este documento son **propuestas técnicas iniciales**, elegidas para hacer el trabajo ejecutable. Pueden evolucionar con un cambio documentado que preserve los requisitos.

No existe todavía un compromiso adoptado con una extensión vectorial, un runtime LLM ni un framework de agentes específico. El perfil local OCR Vision r3/fast/es-ES/en-US está adoptado de forma acotada para imágenes en ADRs 0019/0020; no implica OCR genérico o de PDF. El primer subconjunto PDF usa lopdf =0.45.0 MIT dentro de un helper sin features default: [compilación/árbol/licencias/contención y métricas locales](verification-pdf-extraction.md), no publicidad. Ampliaciones se resuelven en cambios acotados con pruebas de compatibilidad, rendimiento y licencias.

El hardware local se verificó en esta preparación mediante `sysctl`: **Apple M1 Max y 68.719.476.736 bytes de RAM, equivalentes a 64 GiB**. Esto identifica el equipo de referencia; no prueba que cualquier modelo o LoRA funcione con una latencia determinada.

Las capturas originales del sistema de Spotify no estaban disponibles como píxeles en la lectura de los chats. El ejemplo de la sección 11 es una fixture propuesta, no una reconstrucción verificada de esas imágenes. Tampoco se recuperó un archivo de mockup visual; la referencia de diseño queda definida por escrito.

## 4. Requisitos funcionales del producto

### 4.1 Productividad y organización PARA

PARA organiza por propósito: proyectos con resultado, áreas de responsabilidad sostenida, recursos de interés y archivo de elementos inactivos. No se implementará simplemente como cuatro carpetas sin relaciones.

- **Notes:** páginas con título, bloques, propiedades, etiquetas, relaciones, adjuntos y referencias externas.
- **Tasks:** título, estado, prioridad, fecha de vencimiento, fecha planificada opcional, proyecto y notas relacionadas. Subtareas y recurrencias se incorporan después del CRUD básico.
- **Projects:** resultado o propósito, estado, fechas opcionales y vistas de tareas y notas relacionadas. Estados iniciales propuestos: `planned`, `doing`, `done`, `ongoing`, `on_hold`.
- **Tags:** clasificación `area`, `resource` o `topic`; pueden relacionar notas, proyectos, personas y otras entidades. Área y recurso no necesitan identidades duplicadas en tablas independientes.
- **People:** personas relacionadas con notas, proyectos y registros; datos adicionales serán propiedades del sistema.
- **Archive:** estado reversible de cualquier entidad compatible, conservando identidad, contenido y relaciones. Archivar no elimina físicamente el original.
- **Inbox:** captura sin exigir una clasificación previa. Las reglas exactas de salida de inbox se definirán con el cambio de productividad; serán filtros explícitos y reproducibles.
- **My Day y vistas de trabajo:** tareas seleccionadas para el día, pendientes, vencidas, proyectos activos y notas por contexto. No depender de prompts para calcular estados.

Crear contenido desde una vista de proyecto debe establecer la relación correspondiente. Las vistas deben consultar entidades compartidas; mostrar una nota en dos lugares no crea dos copias.

### 4.2 Editor de bloques

El editor crecerá por entregas. Comenzará con texto; después incorporará párrafos enriquecidos, encabezados, listas, checklists, citas, divisores, código, enlaces, imágenes y archivos. Posteriormente: toggles, callouts, tablas simples, columnas y bloques personalizados cuando se especifique su comportamiento.

Cada bloque tendrá identidad estable y tipo. El JSON persistente será semántico y versionado, con adaptadores hacia Tiptap/ProseMirror. Una tabla de contenido enriquecido y una vista de base de datos serán tipos distintos. Una base incrustada guarda una referencia a colección y vista, no una copia de sus registros.

El editor tendrá comandos `/`, navegación por teclado, drag-and-drop donde sea útil, historial de edición y un estado claro de guardado. Un error de persistencia conservará el contenido pendiente en pantalla y permitirá reintentar. Las modificaciones externas nunca deben reemplazar silenciosamente un buffer local.

Markdown seguirá disponible para intercambio. El formato JSON + adjuntos será el mecanismo de exportación fiel; Markdown podrá perder estructuras no representables y deberá advertirlo o conservarlas mediante referencias documentadas. Importar/exportar Markdown no convierte archivos `.md` en la fuente de verdad.

Tiptap dispone de integración documentada con Svelte; la adaptación y el esquema persistente requieren trabajo propio. [Documentación](https://tiptap.dev/docs/editor/getting-started/install/svelte).

### 4.3 Bases de datos y vistas

Las colecciones genéricas permiten construir sistemas nuevos sin modificar código de Noteka. Un registro puede abrirse como página y tener bloques propios.

Propiedades objetivo: título, texto, número, decimal/moneda, checkbox, fecha o rango de fechas, selección, multiselección, URL, email, archivos, relación, fórmula, rollup y timestamps de creación/modificación. Se especificarán coerciones, valores vacíos, cardinalidad y errores por tipo. Cambiar un tipo no debe destruir valores incompatibles sin una migración explícita y recuperable.

Relaciones unidireccionales o con proyección inversa; una única relación lógica evita que se desincronicen dos copias. Rollups iniciales: conteo, suma, promedio, mínimo y máximo sobre propiedades relacionadas. Deben conservar el significado de valores vacíos y errores.

Vistas objetivo: tabla y lista primero; después tablero, calendario y galería. Configuración persistente de columnas, agrupamiento, filtros combinables, ordenamiento, selección y paginación/virtualización. Una misma colección puede tener varias vistas e incrustarse en distintas páginas.

Las reglas de consulta y cálculo viven en Rust. En 3a se adoptan tabla/lista nativas paginadas (50 filas), sin usar una biblioteca de UI como dominio; TanStack Table/Virtual quedan como opciones a evaluar para volúmenes mayores. Ver [ADR 0005](decisions/0005-shared-collection-records.md). Las plantillas se añaden cuando el motor básico sea estable; botones/automatizaciones en fases posteriores con permisos y transacciones explícitos.

### 4.4 Fórmulas y cálculos

El motor será determinista, tipado y ejecutable sin interfaz. Interpretará un AST validado; no ejecutará JavaScript, Rust, shell o SQL arbitrario generado por el modelo.

El primer subconjunto implementado incluye operaciones matemáticas, comparaciones, booleanos, condicionales, acceso a propiedades escalares, rollups y reparto ponderado exacto. Las funciones avanzadas de texto/fechas/listas quedan para cambios posteriores. Ver [contrato y límites](formulas.md), [ADR 0006](decisions/0006-safe-derived-calculations.md). La documentación pública de Notion sirve como referencia de experiencia, no como garantía de compatibilidad. [Referencia](https://www.notion.com/help/formulas).

Se requieren errores identificables de sintaxis y tipos, división por cero, referencias inexistentes y ciclos. El grafo de dependencias debe permitir recalcular afectados cuando cambien propiedades o relaciones. Un error no debe aparecer como un importe válido.

Los importes monetarios usarán decimal exacto o enteros en unidades mínimas, con moneda y regla de redondeo explícitas. Las fechas distinguirán fechas civiles de instantes; los instantes se almacenan en UTC y se presentan con una zona IANA configurable. No asumir que la zona del sistema coincide siempre con la del workspace.

### 4.5 Adjuntos y enriquecimiento

Los originales locales se conservan como archivos administrados, con metadatos en SQLite, hash y referencias desde bloques o propiedades. Importar debe completar la copia antes de publicar una referencia válida; borrar un bloque no elimina automáticamente un archivo usado por otras entidades.

Una cola con ejecución explícita procesa el subconjunto local implementado; la
tabla resume el alcance completo previsto, no todas las capacidades actuales:

| Fuente | Procesamiento previsto | Referencia conservada |
| --- | --- | --- |
| Texto de notas | Extracción semántica, resumen, conceptos y entidades | Entidad, bloque, revisión y rango |
| Imágenes | OCR, descripción si hay modelo visual y metadatos | Archivo y región cuando esté disponible |
| PDF | Texto, páginas, tablas; OCR para páginas escaneadas | Archivo, página y fragmento |
| Excel local | Hojas, encabezados, rangos, celdas y fórmulas existentes | Archivo, hoja y rango |
| Recurso de Drive | Extracción autorizada y caché explícita | Cuenta, resource ID y versión conocida |

La extracción determinista se prioriza antes de usar un LLM. Leer fórmulas de un Excel no implica ejecutar macros ni reconstruir todo Excel. La extracción de texto y la descripción visual son capacidades distintas; si falta una, el sistema debe informar la limitación.

Cada trabajo registra revisión/hash, extractor/version/configuración, estado, intentos y error. Estados implementados `queued/running/succeeded/failed/cancelled/stale`; sin perfil de modelo en estos extractores deterministas. Pausa/cancel/retry/regeneración no modifican originales, publicación cercada contra versiones/ownership viejos. Encolar no ejecuta y run procesa como máximo un adjunto, sin scheduler residente. PDF textual acotado tiene lector por página, XLSX Transitional lector tipado hoja/A1 sin recalcular ni inferir encabezados. La indexación es explícita para UTF-8/PDF y campos persistidos XLSX, con caché/fórmula separados y hojas ocultas incluidas. PDF amplio, OCR de PDF y ampliaciones Excel son entregas posteriores; OCR de imágenes se verificó por separado. Un perfil de modelo sólo se agregará cuando haya uso evaluable autorizado.

Las sugerencias de relaciones se guardan como derivadas hasta que el usuario o una operación autorizada las aplique. Un resumen generado no reemplaza la nota. Los modelos no deben mantenerse cargados constantemente si no hay trabajo.

### 4.6 Búsqueda y respuestas con evidencia

El sistema combinará:

1. Consultas estructuradas sobre entidades, propiedades, fechas y relaciones.
2. Búsqueda textual SQLite FTS5, disponible sin LLM ni índice semántico.
3. Recuperación vectorial de fragmentos, con embeddings configurables.

La búsqueda híbrida aplicará filtros antes o durante la recuperación de forma coherente. Fusionará resultados, eliminará duplicados y respetará un presupuesto de contexto. Reranking es opcional y se incorporará si las evaluaciones justifican su costo.

Cada fragmento lleva entidad, bloque/archivo, revisión y localizador. La respuesta permitirá abrir su origen. Si hay evidencia insuficiente, el agente lo dirá; no presentará contenido inventado como parte de la base. Índices desactualizados, documentos archivados y recursos remotos revocados tienen que distinguirse de contenido vigente.

Una pregunta como «¿Qué pendientes tengo de CRE?» puede requerir consulta estructurada de tareas además de recuperación textual. Ningún índice vectorial debe ser la única fuente de estados o cálculos.

### 4.7 Herramientas y harness de agentes

El harness administra selección de proveedor, contexto, streaming, llamadas a herramientas, validación, límites de ejecución, cancelación, errores y trazas locales. No necesita estar ligado a un framework específico.

El núcleo publica un catálogo de operaciones versionado y esquemas JSON. Las superficies previstas incluyen entidades y bloques, colecciones, propiedades, relaciones, fórmulas, registros, vistas, consultas, validaciones y cambios agrupados.

Nombres orientativos: `entity.create`, `entity.get`, `entity.update`, `entity.archive`, `block.insert`, `database.create`, `property.define`, `relation.create`, `formula.set`, `record.create`, `view.create`, `database.query`, `database.validate`, `changes.preview`, `changes.apply`. Son contratos a definir por fase, no comandos existentes.

Las operaciones deben ofrecer errores útiles, revisión esperada, paginación y resultados estructurados. El núcleo valida y ejecuta; el LLM solicita operaciones. El agente no recibe acceso irrestricto a la base SQLite o al shell para suplir herramientas faltantes.

Un cambio complejo puede prepararse como lote: validar todas sus operaciones, mostrar un diff y aplicar una transacción. Una falla debe dejar el estado anterior completo. El nivel de confirmación depende del permiso concedido, impacto y reversibilidad; las lecturas y acciones ya autorizadas no requieren preguntas repetidas.

El historial de cambios de dominio registra origen, entidades afectadas y revisiones. Deshacer puede crear una revisión compensatoria si no hay conflicto; no se restaurará un snapshot antiguo sobre modificaciones ajenas sin advertencia. No hace falta implementar event sourcing completo para esto.

## 5. Arquitectura propuesta

```text
Svelte / Tauri      CLI de Noteka      MCP local      Harness integrado
       \                 |                |                 /
                    Operaciones versionadas
                             |
                 noteka-core (Rust, sin UI)
              dominio / validación / transacciones
                             |
                  SQLite + originales adjuntos
                             |
         cola de trabajos -> extractores / modelos / índices
                             |
               adaptadores locales y externos
```

Tauri aporta el contenedor de escritorio; las reglas permanecen en una biblioteca Rust que pueda usar un proceso independiente. [Referencia oficial](https://v2.tauri.app/start/).

La primera implementación abrirá SQLite desde la app o CLI, con concurrencia controlada. No requiere un daemon permanente. Si después las colas o el acceso sostenido lo justifican, se podrá añadir un servicio local dueño de las escrituras mediante un cambio específico. El CLI y MCP deben poder funcionar con la ventana cerrada.

Estructura de código adoptada en fases 0–2 y extendida por cambios locales:

```text
src/                       # interfaz Svelte y adaptadores del editor
src-tauri/                 # host Tauri y comandos delgados
crates/noteka-core/        # dominio, contratos, persistencia
crates/noteka-cli/         # comandos y salida JSON
crates/noteka-mcp/         # transporte stdio verificado y archivado
crates/noteka-xlsx-worker/ # helper descartable; cola/lector XLSX explícitos verificados
tests/fixtures/            # datos sintéticos, nunca datos personales
docs/                      # plan, desarrollo y ADRs
openspec/                  # specs vigentes y cambios
```

Propuesta inicial: frontend Svelte + Vite, sin necesidad de SSR. npm y Cargo para dependencias, lockfiles versionados y dependencias concretas elegidas al crear el scaffold. No fijar versiones futuras basándose solo en esta conversación.

### 5.1 Modelo persistente y evolución

| Concepto | Responsabilidad | Autoridad |
| --- | --- | --- |
| Workspace | Identidad, configuración, versión de esquema y zona horaria | Original |
| Entity | ID estable, tipo, título, timestamps, revisión y archivo | Original |
| Document / Block | Documento JSON versionado con bloques identificables | Original |
| Collection / Property / Value | Esquema y valores tipados de registros | Original |
| Relation | Vínculo tipado entre entidades o registros | Original |
| View | Consulta y presentación reutilizable de una colección | Configuración original |
| Template | Definición versionada de página de registro/defaults estáticos, independiente de sus instancias | Original |
| Attachment | Referencia, hash y metadatos del archivo administrado | Original |
| ExternalResource | Proveedor, cuenta, ID remoto y asociaciones | Referencia original; contenido remoto tiene dueño externo |
| Revision / Change | Historial y origen de modificaciones | Registro local |
| DerivedArtifact / Chunk | Extracción, resumen, localizador y versión fuente | Regenerable |
| EmbeddingProfile / Index | Configuración, huella del espacio vectorial y estado | Regenerable |
| Job | Procesamiento pendiente, resultados y errores | Operativo |

El bootstrap implementó workspace, entity, document, relaciones mínimas y cambios. Fase 1 añade attachments/documento v2 sin reescribir v1; fase 2 agrega metadatos civiles y consultas de trabajo, manteniendo IDs. El tramo FTS de 5a agrega índice derivado SQLite v3. No se crearon tablas vacías de fases futuras. Ver [ADR 0001](decisions/0001-local-core.md), [0002](decisions/0002-semantic-blocks-and-originals.md), [0003](decisions/0003-civil-productivity.md) y [0004](decisions/0004-local-text-index.md).

Las entidades built-in pueden convertirse progresivamente en colecciones del mismo motor genérico. Se debe conservar su identidad al hacerlo y evitar un segundo sistema de registros incompatible con notas y tareas. Las propiedades personalizadas no deben exigir una columna SQLite nueva por cada campo definido por el usuario.

Propuesta del documento mínimo:

```json
{
  "schema_version": 1,
  "blocks": [
    {
      "id": "identificador-estable-del-bloque",
      "type": "paragraph",
      "content": [{"type": "text", "text": "Contenido de la nota"}]
    }
  ]
}
```

Los IDs serán opacos para los clientes. Se validarán tipos, bloques duplicados, profundidad y tamaño. Una versión o bloque desconocido no se descarta al guardar: se rechaza la modificación con un error recuperable o se muestra una vista de solo lectura cuando exista soporte para ella.

SQLite tendrá integridad referencial, transacciones y migraciones numeradas. La revisión se incrementará dentro de la transacción. Un cliente que envíe una revisión vieja recibe conflicto, no sobrescritura. WAL y un timeout acotado son decisiones propuestas para permitir app y CLI simultáneos; se probarán ante dos escritores.

### 5.2 Workspace, backup y portabilidad

Los datos de usuario se guardarán en el directorio de datos de Noteka o una ruta de workspace seleccionada, fuera del checkout. Estructura prevista: base de datos, adjuntos, configuración no secreta e índices regenerables.

Un backup fiel incluye SQLite consistente, originales de adjuntos y un manifiesto con versiones y hashes. No copiar a ciegas un `.sqlite` abierto e ignorar su WAL. La restauración valida el backup y crea primero un workspace nuevo; reemplazar datos existentes será una acción separada y explícita.

Git versiona código, documentación, specs y fixtures sintéticas. No versiona notas personales, archivos de ejecución, API keys, tokens OAuth, pesos de modelos ni backups. Los secretos se guardarán mediante el almacén seguro del sistema, con referencias opacas en la configuración.

## 6. Configuración de LLMs y embeddings

La selección de LLM y la de embeddings serán independientes. Cada rol —asistente, enriquecimiento, visión y evaluación— puede usar un perfil distinto.

Un perfil LLM tendrá proveedor/adaptador, endpoint, modelo, contexto utilizable, opciones de generación soportadas, timeout, concurrencia, referencia segura a credencial y capacidades: texto, visión, herramientas, JSON estructurado y streaming. El harness detectará o comprobará esas capacidades; no basta con que un endpoint diga ser compatible con OpenAI.

Los runtimes MLX, llama.cpp, Ollama y LM Studio son opciones a evaluar. La primera integración local puede utilizar un endpoint ya ejecutado por el usuario. No es necesario embutir todos los runtimes ni descargar modelos al iniciar Noteka. Modelos de solo texto pueden utilizar extractores y OCR; los modelos sin tool calling pueden usar un protocolo JSON validado si supera las evaluaciones, o limitarse a consultas sin escrituras.

### 6.1 Preferencias iniciales verificables

- **Qwen3.5-9B:** candidato para uso local habitual. No queda fijado como requisito ni como ganador definitivo de benchmarks. [Model card oficial](https://huggingface.co/Qwen/Qwen3.5-9B).
- **Qwen3.5-4B:** candidato del experimento de herramientas y LoRA. Su repositorio oficial existe; la calidad de operaciones de Noteka se medirá localmente. [Model card oficial](https://huggingface.co/Qwen/Qwen3.5-4B).
- **Jina Embeddings v5 Text Small:** preferencia adoptada para embeddings. La ficha oficial indica 677M parámetros, dimensión máxima 1024 y licencia CC BY-NC 4.0. El usuario aceptó esa licencia para el uso personal previsto. [Ficha oficial](https://jina.ai/en-US/models/jina-embeddings-v5-text-small/).

Para recuperación con Jina se registrará la tarea `retrieval` y se distinguirán entradas `query` y `document`; también se puede evaluar la variante con adapter fusionado. La configuración exacta y revisión de pesos forman parte de la huella del índice. [Model card y ejemplos](https://huggingface.co/jinaai/jina-embeddings-v5-text-small).

No asumir que el nombre de un modelo basta para reproducir resultados: registrar runtime, revisión, cuantización, template y parámetros. Tampoco asumir que un endpoint genérico de embeddings aplica automáticamente los prompts de Jina.

### 6.2 Cambio de embedding y reindexación

El perfil registra proveedor, modelo, revisión, dimensión, normalización, tarea, prompts, cuantización cuando afecte el espacio, tokenizer y versión de chunking. Su huella identifica el espacio vectorial.

Cambiar cualquiera de esos factores crea una nueva generación de índice. No comparar vectores de espacios incompatibles aunque tengan igual dimensión. La reconstrucción se realiza aparte; su activación debe ser atómica tras verificar cobertura y consistencia. Mientras tanto se puede mantener la generación anterior con su perfil de consulta correspondiente, o degradar a SQL + FTS5. Nunca borrar originales para reconstruir embeddings.

Se evaluará una implementación local compatible con Apple Silicon: persistencia, filtros, cancelación, empaquetado Tauri, costo de memoria y métricas de recuperación. La elección del backend vectorial queda para esa fase, con un ADR y benchmark específico; no se da por adoptado uno ahora.

### 6.3 OpenAI, ChatGPT y Gemini

El usuario quiere poder elegir GPT 6.1 Sol mediante su suscripción de ChatGPT y Gemini 3.8 Flash mediante API key, además de modelos self-hosted.

Existe documentación oficial de **Sign in with ChatGPT** para uso de un plan elegible desde aplicaciones. Noteka deberá implementar el flujo oficial, verificar permisos concedidos y consultar los modelos habilitados; un login de identidad por sí solo no autoriza inferencia. La disponibilidad de `gpt-6.1-sol` en la cuenta del usuario no se verificó. No reutilizar sesiones o credenciales internas de Codex como sustituto de esta integración. [Guía oficial](https://developers.openai.com/cookbook/articles/sign-in-with-chatgpt), [condiciones de acceso](https://help.openai.com/en/articles/20001410-sign-in-with-chatgpt).

OpenAI API con key es una modalidad separada y debe mostrarse como tal. El fallo o falta de elegibilidad de ChatGPT no cambiará automáticamente a una API facturada.

Google documenta el identificador `gemini-3.8-flash`. El acceso real depende de la key, proyecto y cuota que el usuario configure. El adaptador debe validar opciones del modelo en lugar de trasladar parámetros de otro proveedor sin comprobarlos. [Ficha oficial](https://ai.google.dev/gemini-api/docs/models/gemini-3.8-flash).

Toda llamada remota deberá hacer visible el proveedor y permitir limitar qué fuentes se envían, presupuesto, tokens y concurrencia. No pasar a la nube automáticamente cuando un modelo local falle. La app local seguirá funcionando sin estas conexiones.

## 7. CLI y MCP

CLI primero: operaciones documentadas, entrada JSON, salida JSON estable, ayuda y catálogo de esquemas. `stdout` contendrá exclusivamente el resultado estructurado en ese modo; logs y progreso van a `stderr`. Códigos de salida distinguirán éxito, validación, no encontrado, conflicto y fallo de almacenamiento.

MCP posterior: tools y recursos mapeados al mismo contrato, sin un segundo conjunto de reglas de negocio. Se comenzará con `stdio` para clientes locales compatibles. Publicar MCP no garantiza que todos los clientes o planes acepten escritura o transporte local. [Documentación MCP](https://modelcontextprotocol.io/docs/2026-07-28/develop/connect-local-servers).

Un ChatGPT remoto no debe suponerse capaz de acceder directamente a un `stdio` del Mac. Acceso remoto, HTTP, túneles, autenticación y permisos se evaluarán en un cambio separado si el cliente elegido los requiere. No abrir un puerto público en la entrega inicial.

La conexión MCP de un agente externo y el proveedor del asistente interno son ejes separados: se puede usar un modelo local dentro de Noteka y otro agente por MCP a la vez.

## 8. Google Drive, Docs y Sheets

Las asociaciones externas serán genéricas: cualquier entidad puede relacionarse con uno o más recursos mediante `provider`, `account_id`, `resource_type`, `resource_id` y metadatos de caché. Por ejemplo, el Tag `CRE`, un Project o una Person pueden tener asignada una carpeta de Drive sin una columna exclusiva por cada tipo.

Un bloque de archivo remoto mostrará nombre, tipo, metadatos, preview disponible y una acción para abrir el recurso original. La carpeta asociada podrá abrirse en el navegador y, cuando haya permisos suficientes, en un explorador modal de Noteka construido con APIs.

OAuth de escritorio usa el navegador del sistema. No basar la autenticación en una WebView embebida de Google. [Documentación oficial](https://developers.google.com/identity/protocols/oauth2/native-app).

Los scopes se definirán por funcionalidad. `drive.file` da acceso acotado a archivos seleccionados/autorizados, y no permite asumir acceso arbitrario a todo el contenido de una carpeta. La navegación profunda y extracción de contenido pueden requerir scopes más amplios y requisitos de verificación. La decisión debe documentarse antes de implementarla; no pedir permisos de escritura por comodidad. [Scopes oficiales](https://developers.google.com/workspace/drive/api/guides/api-specific-auth).

Google Docs: preview mediante una representación obtenida por API/exportación cuando sea posible, y apertura externa. Google Sheets: lectura de rangos o preview disponible, límites claros de filas/hojas y **botón «Abrir en Google Sheets»** en el navegador. No incluir edición embebida ni escrituras sobre celdas.

Referenciar un archivo no equivale a indexar su contenido. La indexación de Drive será opt-in por archivo o conjunto autorizado; registrará cuenta, versión/fecha/hash conocido, extractores y estado de actualización. Las credenciales nunca viajan dentro de un bloque exportado.

Si no hay red o se revoca acceso, mostrar metadatos/caché con su antigüedad, o un error claro; conservar las notas locales. Quitar una asociación no borra el archivo de Drive. Refrescar contenido revocado debe desactivar su recuperación y ofrecer limpieza de la caché. No agregar Google Fotos.

## 9. Interfaz y experiencia

Referencia inicial: Notion oscuro, tipografía legible y controles consistentes. El usuario aportó después siete capturas del **Home que quiere copiar**: portada/nav/clima y secciones Tasks, Notes, Projects, Tags, Goals y People. [Home reference](home-reference.md) conserva composición, pestañas y límites sin copiar datos personales a Git. Las capturas claras no eliminan la opción oscura; no se requiere copiar branding ni controles de colaboración inexistentes.

- Sidebar con Inbox, My Day, Notes, Tasks, Projects, áreas/recursos, People, Archive y búsqueda; elementos futuros solo aparecen cuando funcionen.
- Página centrada y opción de ancho completo. Título, propiedades y relaciones en cabecera, contenido debajo.
- Panel contextual de IA oculto por defecto, con fuentes, operaciones propuestas, progreso y cancelación.
- Tablas y bases incrustadas dentro de páginas, con navegación al registro original.
- Estados explícitos para guardado pendiente, conflicto, procesamiento, índice desactualizado, proveedor desconectado y recurso remoto sin permisos.
- Atajos previsibles, foco visible, contraste suficiente y operaciones esenciales por teclado.

La primera interfaz será deliberadamente mínima: crear, abrir y editar texto, navegar entidades y archivar/restaurar. La presentación completa llegará con el editor y vistas; una carcasa visual no cuenta como funcionalidad implementada.

## 10. Requisitos no funcionales y evaluación

**Integridad:** ninguna escritura con revisión vieja sobrescribe una nueva; un lote inválido no deja modificaciones parciales; un formato desconocido se conserva. Migraciones y backups tienen pruebas de recuperación.

**Privacidad:** el modo local no envía contenido a proveedores externos. Tokens y keys en el almacén seguro, logs locales con datos sensibles minimizados, telemetría remota desactivada salvo decisión explícita futura. No interpretar instrucciones encontradas en notas o archivos como autorización para ejecutar herramientas.

**Recursos:** colas con backpressure, lotes acotados y cancelación; no activar todos los modelos a la vez. Virtualización de listas grandes. Carga y descarga de modelos bajo demanda. Medir la app y runtime de inferencia por separado, incluyendo procesos auxiliares cuando corresponda.

**Objetivos iniciales propuestos de rendimiento:** arranque frío del núcleo/UI menor a 3 s; guardado de una nota de texto de 10 KB p95 menor a 150 ms; búsqueda textual en 10.000 notas sintéticas p95 menor a 300 ms. Se medirán en build de release del M1 Max, con corpus y procesos registrados. Son objetivos de diseño, no resultados comprobados; si no se cumplen, registrar el cuello de botella y revisar el objetivo con evidencia. No bloquear el bootstrap por una función de búsqueda aún fuera de su alcance.

**Pruebas:** dominio y fórmulas deterministas; integración sobre SQLite real; concurrencia app/CLI; contratos JSON; recuperación; pruebas de UI para recorridos centrales; evaluaciones de búsqueda y agentes sobre fixtures. Los mocks verifican contratos; no prueban acceso real a Google ni inferencia real de un proveedor.

**Empaquetado:** primera plataforma macOS Apple Silicon. La distribución, firma, notarización y actualización requieren un cambio propio cuando la aplicación esté lista. El desarrollo local no constituye publicación.

## 11. Recorridos de aceptación de la visión completa

### A. Captura y persistencia offline

Sin red ni LLM configurado: crear una nota, asignarla al proyecto `CRE`, añadir una tarea, cerrar y reabrir. Deben conservarse IDs, bloques, relaciones y archivo reversible. El CLI consulta la misma información con la ventana cerrada.

### B. Respuesta con evidencia

Con notas y PDF de prueba indexados, preguntar por un acuerdo de un proyecto. La respuesta debe citar bloque o página y abrir el origen. Modificar esa nota debe invalidar derivados viejos. Si no hay evidencia, la respuesta debe reconocerlo.

### C. Sistema de suscripción compartida creado por un agente

Fixture sintética: colección `Subscriptions` con costo **Bs 145,69**, moneda BOB y **6 asientos**; colección `Members` con cinco participantes, uno usa 2 asientos y los otros cuatro usan 1. Relación entre ambas colecciones, cálculo de importe, indicador de pago y vista «Pendientes», incrustada en una página.

El agente crea todo usando operaciones públicas, valida fórmulas y muestra resultados antes de aplicar. La suma de importes cobrables debe ser Bs 145,69. Política propuesta para esta fixture: distribuir centavos por restos mayores, con desempate por ID estable; resultado esperado: Bs 48,57 para quien usa 2 asientos y Bs 24,28 para cada uno de los otros cuatro. No usar floats binarios para dinero.

Cambiar el costo recalcula miembros y vistas. Una división por cero, relación inválida o fórmula cíclica da un error identificable. Si el lote falla, ninguna colección queda parcialmente creada. Asientos vacantes, sobreasignación y períodos de pago se especificarán antes de expandir este caso; no inventar reglas financieras ocultas.

### D. Embedding intercambiable

Indexar con Jina, cambiar a otro modelo de dimensión igual o distinta y reconstruir. El documento y sus relaciones no cambian; la búsqueda usa una única generación compatible y puede degradar a FTS5 durante el proceso.

### E. Drive genérico y Sheets sin edición

Asociar la carpeta `CRE` a un tag y a un proyecto. Abrir la asociación y previsualizar una planilla autorizada. El botón abre Google Sheets en el navegador; no hay controles para editar celdas. Repetir sin red y con permiso revocado sin romper las páginas locales.

### F. Modelos configurables y límites

Cambiar entre un proveedor local y uno remoto habilitado conservando las operaciones del núcleo. Un modelo sin visión o herramientas produce una limitación explícita. Fallas de autenticación, cuota, cancelación o JSON inválido no causan una escritura parcial ni un cambio silencioso de facturación.

## 12. Hoja de ruta por cambios OpenSpec

Los nombres futuros son orientativos. No crear todos los cambios ni specs definitivas antes de aprender de cada entrega. Cada fase puede dividirse en más de un cambio si no es implementable y verificable de forma independiente.

| Fase | Cambio sugerido y dependencias | Entrega y salida verificable |
| --- | --- | --- |
| 0 | `bootstrap-local-core` — implementado y archivado | Tauri/Svelte, núcleo Rust, workspace/migraciones/backups, entidades mínimas, nota JSON, CLI y captura UI offline. [Evidencia de sus cuatro capabilities](verification-phase-0.md). |
| 1 | `add-block-editor` — implementado y archivado | Tiptap, adaptador semántico, bloques básicos, guardado con conflicto, adjuntos y exportación fiel/Markdown con límites; [reapertura y recuperación probadas](verification-phase-1.md). |
| 2 | `add-productivity-workflows` — implementado y archivado | Inbox, My Day, tareas con fechas/prioridad, dashboards de proyecto/tag/persona y archivo; [filtros y relaciones probados](verification-phase-2.md). |
| 3a | `add-database-engine` — implementado y archivado | Colecciones, propiedades tipadas, registros/relaciones/vistas tabla/lista; [IDs compartidos verificados](verification-phase-3a.md). |
| 3b | `add-formulas-and-rollups` — implementado y archivado | [AST, tipos, dinero, dependencias y errores verificados](verification-phase-3b.md); fixture de suscripción exacta sin IA. |
| 3c | Dividida: `add-advanced-collection-views` y `add-collection-templates-and-embeds`, ambos archivados — 3a/3b cumplidas | [Primer tramo](verification-phase-3c-views.md): tablero/calendario/galería y filtros complejos. [Segundo tramo](verification-phase-3c-templates.md): templates de páginas con defaults estáticos y referencias de bases incrustadas sin duplicación; 17/17 tareas, documento v3/SQLite v7. Separar evolución de consultas/vistas de evolución de bloques persistentes. |
| Home | `add-local-home-dashboard` implementado, 9/17 tareas verificadas; revisión final cerrada, sin archivar | Composición y quince vistas sobre originales, temas y resúmenes; [606 checks headless y brecha nativa](verification-home.md). UI headless por defecto, pantalla sólo con consentimiento específico. Goals/clima/fotos/auxiliares pendientes, sin paridad total. |
| 4 | [`extend-agent-operations`](../openspec/changes/archive/2026-10-13-extend-agent-operations/proposal.md) implementado, verificado, sincronizado y archivado; 22/22 tareas | Catálogo/CLI de 40 operaciones, preview sin mutaciones de dominio y apply atómico con recibo/huellas; nueve mutaciones y checks database.query. Suscripción/página construidas por lote de 16 pasos y comprobadas offline. Sin migración, LLM ni bloqueo por Home nativo. |
| 5a | Dividida: FTS, cola/UTF-8, PDF **21/21**, índice UTF-8/PDF **19/19**, lector XLSX **8/8**, [extracción XLSX](xlsx-extraction.md) **11/11**, [búsqueda XLSX](xlsx-search.md) **19/19** y [UI de evidencia](evidence-search-ui.md) **17/17**, verificados, sincronizados y archivados | Baseline vigente v13/API v1/53, regresión y dos pares de gates frescos aprobados con evidencia durable; comparación histórica v11 no repetible según autorización. [Evidencia UI](verification-evidence-search-ui.md). Preparación UI 9/9, lector completo por páginas 6/6 y consulta agrupada 6/6 implementados/verificados; [aceptación conjunta](verification-local-preparation-closure.md). Los tres cambios están sincronizados/contrastados y archivados. Lector OCR independiente verificado ([guía](ocr-reader.md)) e [integración de imágenes OCR](verification-image-ocr.md) cerrada12/12, sincronizada y archivada; PDF amplio/OCR de PDF/Excel genérico y resto pendientes; 5a no completa ni dependencia 5b plenamente satisfecha. |
| 5b | `add-semantic-retrieval` — 5a | Adaptador de embeddings, perfil Jina, backend vectorial evaluado, reindexación e híbrida con fuentes. |
| 5c | `add-agent-harness` — fases 4/5b | Proveedor local configurable, QA con evidencia y tool loop acotado; cambio de modelo sin tocar el dominio. |
| 6a | `add-cloud-model-providers` — 5c | Gemini key, OpenAI key y ChatGPT oficial según acceso; capacidades, cuotas, selección de envío y errores probados. |
| 6b | `add-google-drive-references` — fases 1/5a | OAuth, asociaciones universales, previews Docs/Sheets y apertura externa; permisos y estado offline probados. |
| 6c | [`add-local-mcp`](../openspec/changes/archive/2026-10-15-add-local-mcp/proposal.md) verificado 22/22, sincronizado y archivado; dependencia fase 4 cumplida | Recorte inicial stdio: operaciones compartidas habilitadas por política, 23/43/44 herramientas y siete exclusiones explícitas de administración/archivos; catálogo core 51 intacto. SDK real modern/legacy, regresión/revisión y dos gates de consulta/lock aprobados, sin LLM ni pantalla. Puede adelantarse a 5c; no completa toda la paridad futura de 6c. |
| 7 | `evaluate-qwen4b-lora` — fases 4/5c/6c estables | Baseline, dataset y comparación controlada; reporte de aciertos, errores, latencia y memoria. No bloquea uso normal. |
| 8 | Cambios separados futuros | Sincronización entre Macs y versión web, después de uso local estable; diseño de conflictos/autenticación propio. |

**Primera entrega utilizable:** completar fases 0–2 permite notas y productividad local. **Entrega que cumple la visión principal:** fases 3–6 permiten sistemas genéricos y agentes intercambiables con evidencia. LoRA es una investigación adicional. No se fija una fecha ficticia ni se promete que todo sea un MVP pequeño.

**Recorte actual de5a:** `add-local-image-ocr` tiene implementación y aceptación
técnica aprobadas: representación decimal/migración13, lector de regiones y
preparación/selección/búsqueda explícitas. SQLite13/API1/53, MCP25/45/46 y batch9.
Pasaron319 Rust por perfil,3442 checks headless, oráculos/procesos/MCP completos
en ambos perfiles y cuatro gates finales (query máximo254,54 ms, lock18,81 ms).
Doce tareas verificadas; cinco deltas sincronizados/contrastados y
[archivo](../openspec/changes/archive/2026-10-17-add-local-image-ocr/tasks.md) comprobado. [Guía](image-ocr.md),
[evidencia](verification-image-ocr.md), [revisión](review-image-ocr.md) y
[ADR0020](decisions/0020-local-image-ocr.md). PDF amplio/Excel y el cierre de5a
requieren sus cambios propios; no iniciar5b antes de cumplir dependencias.

La trazabilidad del primer cambio es explícita: `local-workspace` cubre persistencia/backup; `knowledge-entities` cubre entidades/documentos/relaciones/archivo; `core-operations` cubre contratos/concurrencia/CLI; `desktop-shell` cubre navegación/captura/guardado. La búsqueda, editor rico y demás requisitos se formalizan al proponer su cambio.

## 13. Experimento Qwen3.5-4B + LoRA

Primero medir el modelo base contra un conjunto de tareas representativas: crear notas, definir esquemas, establecer relaciones, escribir fórmulas, construir la suscripción, recuperar citas, resolver errores y respetar permisos. Comparar con Qwen3.5-9B y un modelo externo disponible bajo las mismas herramientas y fixtures.

Métricas: éxito de tarea completa, argumentos válidos, consistencia final, cálculos exactos, citas verificables, mutaciones indebidas, cantidad de llamadas, latencia, memoria y costo remoto cuando aplique. Un benchmark publicado por un proveedor no sustituye este reporte.

El dataset LoRA usará ejemplos sintéticos o contenido autorizado, herramientas versionadas, casos de error y corrección. Separar train/validation/test por familias de tareas y estructuras, evitando que variantes de un mismo sistema contaminen el test. Conservar semillas, templates, revisiones y condiciones de ejecución.

La viabilidad de entrenar Qwen3.5-4B en el M1 Max depende del soporte real del runtime para su arquitectura, del tamaño de contexto y de los parámetros del ajuste. Medir un experimento pequeño antes de estimar entrenamiento completo. Si requiere otro equipo, sigue siendo una investigación opcional.

El ajuste no incorpora secretos ni conocimiento personal como sustituto de retrieval. Se versiona adapter y dataset autorizado, se evalúa después de cambios en herramientas y se compara contra el baseline sin alterar condiciones. Si no mejora o perjudica el respeto de límites, se conserva el modelo base.

## 14. Riesgos y decisiones diferidas con límites

| Riesgo o dependencia | Tratamiento y momento de resolverlo |
| --- | --- |
| Alcance parecido a Notion demasiado amplio | Cambios pequeños; primero núcleo, captura y productividad. No aceptar una promesa de paridad completa. |
| Documentos atados al formato del editor | Modelo semántico propio, adapters y migraciones; resolver antes del editor. |
| Dos modelos incompatibles de entidades/colecciones | IDs compartidos y migración de built-ins; resolver en 3a antes de nuevas propiedades. |
| Fórmulas lentas o cíclicas | AST tipado, grafo de dependencias y límites de complejidad; resolver en 3b. |
| Corrupción o pérdida por múltiples clientes | Transacciones, revisión esperada y backup probado desde fase 0. |
| Jina o su runtime no cumplen requisitos locales | Probar adapter y perfil; conservar configuración intercambiable. No sustituir preferencia silenciosamente. |
| Google scopes/acceso insuficientes | Matriz de permisos y recorrido real antes de declarar integración completa. Apertura externa sigue disponible. |
| ChatGPT no habilitado para la cuenta o modelo elegido | Modalidad condicionada a permisos/modelos devueltos; no usar alternativas facturadas automáticamente. |
| Capturas o modelo de datos anterior incompletos | Fixture de suscripción explícita; validar reglas del caso real antes de una importación fiel. |
| Sincronización prematura | IDs y revisiones permiten evolución, pero no elegir CRDT o servidor remoto ahora. |

Antes de cada fase, revisar documentación oficial vigente. No recalcular todo el plan por rumores de modelos nuevos: el requisito es poder cambiarlos. Preguntas futuras sobre compatibilidad exacta con Notion, alcance de permisos Google o transporte MCP remoto afectan esas fases; no bloquean el bootstrap local.

## 15. Instrucciones para la próxima IA

1. Leer este documento completo, `AGENTS.md`, `openspec/config.yaml` y `docs/openspec-workflow.md`.
2. Inspeccionar archivos y Git; este estado inicial es documental, pero puede haber evolucionado. Resolver la raíz con `openspec context --json` y comprobar specs y cambios existentes.
3. Fases 0–4, FTS de notas, cola local/UTF-8, PDF textual acotado, índice explícito UTF-8/PDF/XLSX y extracción/lector XLSX verificados, sincronizados y archivados; baseline vigente v12/API v1/52. `add-local-xlsx-search` cerró 19/19 el 16/10, con cinco deltas sincronizados/contrastados; leer `verification-xlsx-search.md` y `xlsx-search-evidence-recovery.md`. La pérdida del baseline se resolvió mediante criterio alternativo autorizado, no recuperándolo ni fabricando v11; comparación exacta histórica no repetible queda explícita. `add-local-evidence-search-ui` cerró 17/17 con su único delta sincronizado/contrastado y archivado: ámbitos separados, evidencia textual y cobertura verificadas; leer `evidence-search-ui.md` y `verification-evidence-search-ui.md`. No recrear esos cambios. Home tiene código/revisión y 606 checks headless, con 9/17 tareas verificadas; leer `verification-home.md` y `review-home.md`. Sus ocho tareas nativas no se completan con el puente: no iniciar Tauri ni capturas del escritorio sin nueva autorización de pantalla. Los cambios independientes continúan por dependencias sin rebajar Home. Separar Goals/fotos/clima/auxiliares. Preparación UI/lector completo por páginas/consulta agrupada tienen aceptación conjunta documentada; sus cierres se registran por cambio. PDF amplio/Excel/OCR y resto de 5a pendientes. No pedir autorizaciones repetidas de desarrollo local; registrar revisiones externas en la cola.
4. Implementar por tareas verificables. Registrar versiones reales del scaffold y comandos reales de desarrollo en `docs/development.md` cuando existan, y mantener lockfiles.
5. Ejecutar validaciones del cambio y contrastar cada escenario con código y evidencia. Marcar tareas realizadas; dejar pendientes y limitaciones explícitas.
6. Verificar y archivar solo cuando el comportamiento del cambio esté implementado y validado; sincronizar entonces los deltas a specs principales. No archivar esta propuesta porque sus documentos existan.
7. `add-local-pdf-text-extraction` está archivado 21/21; leer su verificación antes de ampliar parser/cola/lector. `add-local-attachment-search` está archivado como `2026-10-14-add-local-attachment-search`, **19/19 tareas verificadas** y sus tres deltas sincronizados/contrastados: dos benchmarks de query p95 214,20–231,07 ms sobre el corpus fijado (<300 ms), lock aprobado y scratch físico medido, además de regresión/revisión. No recrearlo ni volver a sincronizarlo. Leer sus artefactos archivados, guía/ADR 0013 y evidencia externa, incluidos fallos previos, reconciliación de migración y límites por corpus/caché. No se cambió objetivo/caps ni se retiró validación; no garantiza 300 ms universal. No sincronizar/archivar Home ni declarar 5a/5b completas. Actualizar este plan y ADRs ante decisiones duraderas.

La fase 4 está implementada, verificada, sincronizada y archivada en `2026-10-13-extend-agent-operations`, con 22/22 tareas y matriz de diez requisitos/29 escenarios. Las specs principales ya incluyen preview/apply. No recrear el cambio ni confundir artefactos completos con tareas verificadas. Los recibos implementados son comparación de estado, no permisos/firmas ni idempotencia persistente; preview usa copia de evaluación, apply nunca reemplaza la base original por ella. Extracción, harness/MCP y modelos siguen siendo cambios posteriores, no scope encubierto de fase 4.

Propuesta de instrucción de arranque para otro agente:

> Leé AGENTS.md, el plan y las verificaciones. Fases 0–4 y FTS archivados; Home implementado, cambio activo sin cerrar. Elegí el siguiente cambio por dependencias de la hoja de ruta y mantené pruebas UI headless, sin usar la pantalla del usuario ni afirmar cobertura del host nativo con el puente CLI. Documentá lo que requiere intervención, no pidas revisiones rutinarias ni publiques/envíes datos sin autorización específica.

## 16. Antecedentes y fuentes

Las decisiones de producto se recuperaron de los chats [Archivos Markdown o vectorización](chatgpt-conversation://6ac95447-3f08-83e9-8ab8-def59f64de23) y [Usar OpenSpec Al Iniciar](chatgpt-conversation://6acc47d6-0454-83e8-9c76-9f2c73670fe9). Este archivo incorpora lo necesario para no depender de acceso a esos enlaces.

Las fuentes oficiales enlazadas junto a las afirmaciones se consultaron durante esta preparación. Describen disponibilidad técnica, no acceso verificado de la cuenta del usuario ni pruebas de Noteka. Los modelos y servicios deben revalidarse al implementar su fase.


**Lector PDF nativo verificado (17/10):** `add-bounded-pdfkit-reader`,6/6 tareas,
sincronizado/contrastado y archivado como2026-10-17-add-bounded-pdfkit-reader. Protocolo Rust privado y helper PDFKit
independiente, sin DDL, perfil público o integración con cola/API/MCP/FTS/UI.
22 archivos propios,13 wires y24 procesos verificados por perfil;326 Rust por
perfil. Cuatro gates consecutivos aprobados: query p95 máximo267,457625 ms y
lock22,331125 ms frente a300/150. Dos intentos mixtos fallidos e investigación
preservados, sin causa demostrada ni cambios de cuotas/validación/objetivos.
Ver [lector](pdfkit-reader.md), [verificación](verification-pdfkit-reader.md) y
[ADR0021](decisions/0021-bounded-pdfkit-reader.md).
Esto no completa5a: integración del perfil nativo, OCR dePDF y ampliaciones
Excel siguen pendientes. Home activo9/17 no participa. Límite vigente de5a:
18/10 02:16 UTC (17/10 22:16 Bolivia); detener de forma segura e informar si sigue
abierta, sin reiniciar el contador.


**Integración PDFKit en curso:** `add-local-pdfkit-extraction`,2/8 tareas
verificadas: baseline542/22, fixtures reales13 con backup/restore exacto, consumidor
Rust y codec canónico privado. Siete tests por perfil y22 PDF reales por perfil,
Clippy/fmt aprobados.014, persistencia/cola, lectura compartida/API54/MCP y packaging
siguen pendientes; el codec es parte parcial de3.1, no persistencia terminada.
Producto conservado en SQLite13/API53/MCP25/45/46/batch9;22 binarios y todo el
catálogo anterior exactos, Home9/17 intacto. Ver [estado](pdfkit-extraction.md).
No se sincroniza ni archiva este cambio parcial. Deadline5a sigue02:16UTC.
