# Design

## Context

Ver motivación en `proposal.md` y referencia confirmada en `docs/home-reference.md`. Estado **inspeccionado**: no existe Home; SQLite v7/documentos v1–v3 y 37 operaciones están implementados. Este documento describe adiciones **propuestas**, no capacidades disponibles.

La inspección anterior corresponde al baseline previo a implementar este cambio. Estado al 13/10/2026: Home implementado y revisión final cerrada; aceptación nativa pendiente, sin sincronizar ni archivar. Resultados y trazabilidad en `docs/verification-home.md` y `docs/review-home.md`; no se sustituyen los criterios de aceptación de este diseño por pruebas del puente headless.

Fronteras comprobadas:

- `App.svelte:14–25,93–125`: secciones existentes, apertura inicial en Notes y consultas `entity.list`/`workflow.query`. `73–84,168–209` concentra guardias, lectura vigente al abrir y guardado/captura; Home debe integrarse ahí, no copiar la lógica del editor.
- `productivity.rs:61–129`: `workflow.query` devuelve entidades + cursor + día; My Day usa planned_date, Overdue due_date < día y Active Projects incluye planned/doing/ongoing. No contiene Today, proyección ligera, grupos ni resúmenes.
- `contracts.rs:119–208`: Note sólo tiene inbox, Person payload vacío, Tag area/resource/topic y cinco estados Project. Relaciones de proyecto unen Note/Task→Project; tag/person son vínculos explícitos (`entities.rs:500–517`). No existen favoritos, organización, fotos ni Goals.
- `entities.rs:65–85,175–262`: `get` carga documento completo; `create_in_transaction` comparte entidad/contexto/registro/historial/FTS bajo Immediate. `EntityFields.svelte` ya edita fechas/estado/clasificación sin controles para fotos/Goals.
- `style.css:1,57–89`: tema oscuro fijo y shell de tres columnas. `CollectionPresentation` consume `DatabasePage` de colecciones, no los payloads built-in. `embedded-reads.ts` dispone de cola/lifetime que descartan respuestas sin liberar prematuramente un IPC.
- `tests/productivity.rs:103–233` verifica filtros previos, fechas civiles, contexto y paginación; `tests/native/smoke.js` y `scripts/native-smoke.mjs` cubren UI/CLI/SQLite reales offline en tres sesiones. Se extienden, no se sustituyen por capturas o mocks.

## Goals / Non-Goals

**Goals:** una proyección local compartida por clientes; ninguna segunda identidad ni escritura por leer; Home fiel en jerarquía y presentaciones con cobertura parcial explícita; consultas y resúmenes consistentes por sección; conservación de todos los recorridos actuales y buffers.

**Non-Goals:** convertir built-ins en colecciones automáticas, crear un documento Home editable, ampliar el AST de fórmulas, mantener snapshots entre llamadas, implementar submódulos diferidos o añadir dependencias remotas. Las demás exclusiones están en proposal. Las reglas aquí elegidas son de Noteka salvo Today, cuyo filtro de referencia sí se inspeccionó.

## Decisions

### 1. Una consulta de proyección por vista, no cinco sistemas nuevos

Añadir `home.query` en catálogo/dispatcher y `noteka home query` en CLI. Contratos estrictos en `home_contracts.rs`, lector en `home.rs`, reexportados por contracts/lib. Se reutilizan enums, validación civil y los datos de productividad; no se modifica `WorkflowView` ni el comportamiento de operaciones previas.

Input: `{view, as_of?, limit?, cursor?}`; `view` enum cerrado de 15 valores:

`tasks_today`, `tasks_inbox`, `tasks_all`, `tasks_done`, `notes_all`, `notes_inbox`, `projects_active`, `projects_planned`, `projects_board`, `tags_recent`, `tags_alphabetical`, `tags_types`, `people_contacts`, `people_new`, `people_updated`.

`as_of` admite fecha civil canónica 0001–9999. Sin cursor ni as_of, Rust resuelve el día en la zona IANA del workspace usando el mismo helper que workflows. Con cursor y sin as_of, mantiene el día del cursor; con as_of distinto lo rechaza. Límite nuevo 1–100, default/UI 50; no reutilizar sin ajuste `entities::limit`, cuyo máximo vigente es 200.

Salida `HomePage`: vista, día/zona, `items`, `groups`, `matching_count`, `unavailable_count`, offset y `next_cursor`. Cada item contiene referencia ligera de entidad (ID/tipo/título/revisión/timestamps/archivo), metadatos de trabajo conocidos con estado de disponibilidad, referencias de contexto y resumen específico (proyecto/tag). **No incluye documento, JSON de payload, adjuntos ni contenido de notas**. Para recuperar/editar se usa `entity.get` original al abrir.

La antigüedad de filas se presenta desde timestamps originales; si se ofrece «hace N días», el entero civil firmado se calcula en Rust con la zona/día de consulta. Timestamp inválido produce no disponible y fecha futura se identifica, no se disfraza como antigüedad cero. No se infiere una clasificación de nota: las etiquetas visibles son relaciones a tags reales.

**Alternativas:** cinco llamadas `entity.list` seguidas de `relation.list/get` por fila generan N+1, cargan documentos y carecen de snapshot de resumen; construir colecciones espejo añade configuración/identidad implícita; ampliar workflows con un resultado incompatible compromete clientes previos. La operación de lectura dedicada evita esas fronteras sin cambiar dominio persistente.

### 2. Reglas de las quince vistas y composición de página

Todas excluyen la entidad principal archivada. Un contexto archivado permanece visible con badge, ID y apertura del original. Estos son filtros **propios**, no una reconstrucción de pestañas no inspeccionadas:

| Vista / etiqueta UI | Selección | Orden / presentación |
| --- | --- | --- |
| tasks_today / Today | state todo y due_date <= día; due_date ausente no coincide | due_date asc, created_at asc, ID asc; lista sin grupos, como inspección autenticada |
| tasks_inbox / Inbox | state todo e inbox true | created_at asc, ID asc; grupos por proyecto |
| tasks_all / All | Todas las tareas activas | created_at asc, ID asc; grupos por proyecto |
| tasks_done / Done | state done | updated_at desc, ID asc; grupos por proyecto; accesible desde menú de vistas adicionales |
| notes_all / Notes | Todas las notas activas, estén o no en Inbox | updated_at desc, ID asc; filas compactas |
| notes_inbox / Inbox | Notas con inbox true | created_at asc, ID asc; filas compactas |
| projects_active / Active | doing/ongoing | rango de estado doing→ongoing, título normalizado, ID asc; grupos por estado |
| projects_planned / Planned | planned | título normalizado, ID asc; lista |
| projects_board / Board | Todos los proyectos activos, incluidos done/on_hold | planned→doing→ongoing→done→on_hold, título normalizado, ID asc; cinco columnas |
| tags_recent / Recent | Todos los tags activos | **updated_at del tag** desc, ID asc; lista, no ordenar por última nota |
| tags_alphabetical / A–Z | Todos los tags activos | título normalizado, ID asc; lista |
| tags_types / Types | Todos los tags activos | area→resource→topic, título normalizado, ID asc; grupos |
| people_contacts / Contacts | Todas las personas activas | título normalizado, ID asc; galería textual |
| people_new / New | Todas las personas activas | created_at desc, ID asc; galería, no «creadas esta semana» |
| people_updated / Updated | Todas las personas activas | updated_at desc, ID asc; galería, no «actualizadas recientemente» como filtro |

Título normalizado = `to_lowercase` Unicode de Rust; igualdad se desempata por ID bytewise asc. No prometer collation española/Notion ni agregar ICU. Etiqueta Types usa **Area/Resource/Topic**, no Entity: topic se conserva, no se migra para imitar la captura. La explicación de Active indica «Doing/Ongoing»; el acceso lateral Active Projects **sigue incluyendo Planned**, como antes. My Day/Overdue tampoco cambian.

Los grupos contienen claves tipadas, etiquetas y `entity_ids` de la página, no objetos repetidos. Tareas con varios proyectos aparecen en varios grupos con el **mismo ID**; encabezado indica elementos únicos y cada grupo sus apariciones de página. Sin proyecto es grupo explícito. Se ordenan proyectos de grupos por título normalizado/ID; se conservan incluso archivados porque no se borra el vínculo. En Board se incluyen los cinco estados vacíos; en Active los dos admitidos, y en Types los tres tipos vacíos. Cantidades de grupos son de página, no de toda la vista.

Paginación reemplaza la página con **Siguiente / Inicio**, no acumula todo el workspace. Se elige una página global de IDs únicos antes de construir grupos. Puede haber columnas vacías en una página aunque otra tenga registros: el texto de cobertura lo explicita. Sin escrituras concurrentes, todas las páginas recorren el mismo orden. Con escrituras, refrescar vuelve al inicio y no se promete snapshot global.

### 3. Snapshot de lectura, resúmenes exactos y límites

Cada `home.query` usa una transacción de **lectura Deferred** para metadatos, selección, relaciones y resúmenes, sin locks de escritura innecesarios ni `entity.get` que expanda documentos. Consultas parametrizadas/fragmentos estáticos del enum; no SQL del cliente. Una sentencia de adquisición inicial establece el snapshot; cambios de CLI posteriores se reflejan en la próxima llamada, no mezclando revisiones dentro de la respuesta.

Lectura por conjuntos/batches: filas candidatas ligeras del tipo de vista; vínculos de contexto de la página; tareas de los proyectos paginados; notas de los tags paginados. No recorrer `database.query`, vistas incrustadas o fórmulas para calcular resumen built-in. Relaciones cuentan por entidad original distinta, con validación de orientación/tipo; documentos futuros no impiden proyectar identidad/metadatos conocidos.

Por proyecto: tareas no archivadas vinculadas directamente con relation.kind project. `total`, `done`, `pending` y `overdue` (todo y due_date < día), porcentaje entero `floor(100 * done / total)` mediante enteros comprobados. Total cero → porcentaje null y «Sin tareas», no 0% ni 100%. Una tarea ligada a dos proyectos cuenta una vez en cada uno, sin duplicarse dentro de un proyecto. No aplicar este progreso a Goals ni al estado del proyecto, ni copiar una supuesta fórmula Notion.

Por tag: todas las notas no archivadas con vínculo tag, conteo distinto y última por `(updated_at desc, ID asc)`. Ninguna nota → count 0 y last null. No confundir esta última nota con el orden Recent del tag. Los resúmenes abarcan todas las fuentes pertinentes, no la página mostrada de Tasks/Notes, y no incrementan revisiones.

Lectura conservadora de payloads: no reutilizar el parser **de escritura** `productivity::payload`, que niega campos futuros. Leer sólo claves conocidas con tipos validados, conservar extras sin conversión. Payload no objeto es no disponible; ausencia legítima **en objeto** heredado respeta defaults actuales (Task todo/normal/inbox true, Project planned, Note inbox true); null es permitido en fechas opcionales, no significa estado/default válido. Valor presente incompatible → disponibilidad explícita. Vista filtrada que no puede decidir pertenencia no incluye el item como match y expone `unavailable_count`; vistas All/Board/Types pueden mostrar su identidad con metadatos no disponibles y grupo separado «No disponible». Estado/vencimiento inválido de una tarea relacionada deja resumen del proyecto no disponible, no conteos/porcentaje aparentemente completos. Metadatos desconocidos **no relevantes** no invalidan el resto de una fila.

Límites propuestos comprobables: máximo 10.000 candidatos principales por consulta, 10.000 fuentes de resumen distintas y 50.000 vínculos pertinentes; lectura acotada con `LIMIT máximo+1` antes de materializar/sortear, error `validation_error` con detalle de límite al superarlo, nunca resumen truncado como completo. Contexto enviado: máximo 50 referencias por fila, orden kind/título normalizado/ID y `context_total` para mostrar «+N»; el resto se consulta al abrir original. Grupos usan **todos** los proyectos asociados, no sólo esos chips; máximo 200 grupos por página o error explícito. Payloads se leen por claves necesarias, no un JSON futuro arbitrariamente grande. Estos límites no limitan el CRUD ni la búsqueda existentes.

Cursores nuevos y estrictos: string JSON de versión 1, scope de workspace (ID + huella de ruta canónica), view, día, offset y límite; máximo 2 KiB. Rechazar versión/campos/tipos/scope/offset/límite incompatibles; no aceptar cursores entity/workflow. El día del cursor evita cambiar filtro al pasar medianoche. No es token firmado ni mecanismo de seguridad: scope previene mezcla accidental. Un restore en otra ruta no hereda un cursor de pantalla anterior.

**Alternativas:** contar sólo filas visibles es incorrecto; persistir porcentajes exige invalidación y migración sin necesidad; fan-out de consultas por cliente mezcla snapshots. El límite de candidatos se acepta como el motor avanzado actual, pero se medirá específicamente y se documentará como capacidad acotada, no como soporte ilimitado.

### 4. Home como ruta de presentación integrada al editor

Agregar `home` a Section y sidebar; tras abrir/crear workspace, entrar en Home. Su shell ocupa sidebar + ancho restante, **sin la columna entity-list vacía** y sin el max-width de la página de editor. Fuera de Home permanecen navegación/listados/editor anteriores. No crear una entidad «Home» ni almacenar su DOM/filas en document v3.

`HomePanel.svelte` controla vistas/páginas y componentes de presentación por sección; Rust entrega grupos/resúmenes y TypeScript sólo da formato/render. No reutilizar `CollectionPresentation` mediante conversiones ficticias a `DatabasePage`; mantener contratos propios de built-ins y reutilizar únicamente tokens/componentes visuales compatibles. Tampoco tocar el NodeView incrustado.

Composición: portada abstracta **propia** local (CSS/SVG azul/verde), casa superpuesta, título Home, menú compacto Tasks/Notes/Projects/Tags que navega a las pantallas reales, franja clima pendiente y seis secciones. Content en información de cobertura, no enlace falso. Cabeceras azul suave, tabs en píldora, contexto/fechas a derecha, grupos plegables mediante controles nativos, Board de cinco columnas y tarjetas People uniformes **sin imágenes**. En ancho menor el contenido refluye/board hace scroll acotado, no oculta controles.

No descargar portada/fotos ni reutilizar assets privados de Notion. Fotos requerirán referencia local autorizada y modelo en otro cambio. Cada pestaña de referencia no implementada se enumera en un detalle «Cobertura pendiente», no botones de vista inertes ni porcentajes de ejemplo:

| Zona | Este cambio | Sigue pendiente para paridad |
| --- | --- | --- |
| Portada/nav/tema | Jerarquía y acceso local, claro/oscuro | Portada personalizada, Content |
| Clima | Franja y aviso «Clima no implementado; no se realizan consultas remotas» | Configuración de localidad/proveedor, opt-in, estado cache/offline, siete días |
| Tasks | Today/Inbox/All/Done, fechas/contexto y grupos donde corresponde | Week/Month/Scheduled/No Due/Recurring y vistas por proyectos/content con reglas propias |
| Notes | Notes/Inbox, contexto, antigüedad y páginas | Fav./Clips/Voice/Journal/Meetings/All diferenciada |
| Projects | Active/Planned/Board, estados y resumen determinista | Reglas equivalentes a toda fórmula/campo de Notion no inspeccionado |
| Tags | Recent/A–Z/Types, conteo/última nota | Fav.; tipo Entity no se simula renombrando Topic |
| Goals | Cabecera/sección pendiente en posición solicitada | Entidad, estados, relaciones, fechas/progreso y vistas |
| People | Contacts/New/Updated, tarjetas y tags/proyectos directos reales | Fotos, organización, Check-Ins/Birthdays/Pipeline/Active Projects |

La verificación compara capturas **sintéticas** de la app con la descripción genérica, registra diferencias y no afirma «1:1» mientras haya zonas pendientes. Los filtros no inspeccionados no se investigan leyendo notas privadas ni se dejan como pregunta bloqueante: las vistas propias están especificadas y su diferencia se informa.

### 5. Apertura y captura: una sola guardia y un solo commit

Callbacks Home a App reciben **ID** (no objeto snapshot para editar) o `{kind, context?, seed?}` para comenzar buffer. App usa `guarded`, vuelve a leer `entity.get` y entra al listado/editor de ese tipo antes de abrir/cargar draft. Captura usa `startDraft`, compartido con `newEntity`, ampliado con defaults tipados de presentación y ruta adecuada; no duplica `save`, no construye escrituras en el componente Home y evita guardias reentrantes al cambiar Section.

El bloqueo de operación abarca desde la primera lectura de apertura/captura, no sólo el fetch final del editor: un atajo o navegación competidor no puede crear un buffer que esa lectura limpie después. Al preparar una captura se libera el bloqueo antes de crear/enfocar el draft. Es integración de la guardia existente, no otro commit ni una regla nueva de productividad.

Capturas: cabecera por tipo; Task desde grupo Project pasa `ContextLink` original, Sin proyecto no pasa context; columna Board pasa sólo Project state; grupo Types pasa Tag classification. Today abre Task sin asignar due_date/planned_date. No marcar done ni salir de Inbox automáticamente por elegir una vista. Completar/reclasificar sigue en el editor actual con revisión esperada, no checkbox inline que parezca una checklist persistida.

Entrar en Home desde entidad/colección pendiente protege guardar/descartar/cancelar y el cierre nativo. Cambiar pestaña/plegar grupo no crea buffer; iniciar una captura sí. Si el target desaparece/falla/está archivado, se identifica el estado sin tomar una copia como original; target archivado conserva el comportamiento actual de apertura/relación, no hay desarchivo automático.

Después de commit confirmado, limpiar pending/estado antes de lecturas secundarias. Lectura Home fallida tiene reintento de lectura, no repetición de creación ni supuesto rollback del commit. Los tests anteriores de conflicto, recuperación y foco no se debilitan por el nuevo shell/default inicial.

La captura abre el editor existente; guardar mantiene ese original abierto. **Volver a Home** es navegación protegida, no commit adicional. El caso guardar y continuar hacia Home puede confirmar la creación y fallar sólo su consulta posterior: la pantalla informa lectura fallida, con el buffer de creación ya limpio y el ID confirmado, sin «Reintentar creación».

### 6. Estado de lecturas con backpressure y generación

Cada sección mantiene view, resultado de **una** página, cursor, busy/error, generación y lifetime. Al cambiar view/página/día o refrescar se retira la proyección previa y se inicia generación; sólo la respuesta con la misma identidad workspace/view/as_of/cursor puede publicarse. Un fallo deja la sección sin filas atribuidas a esa nueva consulta y ofrece reintento; otras secciones mantienen sus propias selecciones/resultados.

Las cinco lecturas iniciales pasan por una `ReadQueue(2)` de Home; reutilizar la utilidad existente sin modificar su significado. Encoladas canceladas no ejecutan; IPC iniciado conserva cupo hasta resolver aunque se descarte. Desmontar cancela lifetimes/observadores/listeners y no actualiza App después. La limpieza se retorna **sincrónicamente** desde onMount; no `onMount(async () => cleanup)` ([documentación oficial de Svelte 5 consultada](https://svelte.dev/docs/svelte/lifecycle-hooks)).

Leer inicialmente, al pedir Actualizar/Inicio/Siguiente/cambiar vista y al volver a activar la ventana; no polling/timers permanentes. App refresca Home vía un controlador como DatabasePanel, sin delegar la misma activación a dos listeners que dupliquen llamadas. Cada sección muestra su propio día/zona de consulta; llamadas independientes **no constituyen un snapshot único de todo Home**. Páginas pasan el día del resultado/cursor; refresh al inicio vuelve a resolver día actual. No afirmar actualización en vivo entre consultas ni mezcla de días como si fuera una sola lectura.

### 7. Tema local y accesibilidad, no configuración de dominio

Oscuro inicial conserva preferencia previa; selector claro/oscuro afecta shell y controles de navegación/editor/Home mediante tokens CSS `data-theme` de raíz. Sustituir hardcodes de superficies/texto/bordes/foco/errores necesarios de `style.css`, sin cambiar reglas ni colores que signifiquen estados de dominio. No añadir librería de UI, React ni upgrade de TypeScript.

Preferencia `noteka.theme.v1` en almacenamiento local de WebView, sin notas/rutas/credenciales; errores de lectura/escritura se capturan, valor inválido usa oscuro. Tema activo se puede cambiar en memoria aunque persistir falle, con aviso de preferencia no guardada. No es dato de SQLite ni entra al backup del workspace. Vistas/grupos/scroll son estado efímero por ruta; se reinician al cambiar workspace, no se prometen preferencias guardadas de vistas.

Objetivos de aceptación: contraste de texto normal >=4,5:1 y foco/controles >=3:1, nombres/estado de selección accesibles, Tab/Enter/Espacio para controles y foco tras guardia/apertura. Tabs pueden implementarse como botones con `aria-pressed` y navegación Tab; no adoptar roles tablist sin interacción completa. Grupos usan details/summary; no usar controles sólo-hover. Capturas 1920×845 de referencia y 1280×800/900×800 para comprobar reflujo, con datos largos sintéticos. No equivale a certificación WCAG ni prueba de lectores de pantalla/periféricos físicos.

## Risks / Trade-offs

- [Primer Home parcial] → matriz de cobertura, avisos Goals/clima y ausencia de botones falsos; cambios siguientes conservan sus posiciones, no declarar toda la referencia concluida.
- [Active/Today se confunden con workflows anteriores] → ayudas/reglas visibles y tests de conjuntos distintos; no modificar My Day, Overdue ni Active Projects previos.
- [N+1, documentos pesados y conteos sólo de página] → proyección ligera, batches y snapshot por respuesta; límites explícitos, medición release en M1 Max con 10.000 entidades y vínculos sintéticos.
- [Escrituras durante paginación] → orden y scope estable sin concurrencia, aviso de snapshot por llamada, refresh al inicio; no prometer cursores congelados.
- [Payload futuro válido pierde campos] → lectura por claves conocidas/no writes; disponibilidad explícita, recuperación con entity.get y pruebas de bytes/revisiones/FTS/historial.
- [Refresh tardío roba foco o publica después de navegar] → lifetime/cola y espera de controles habilitados; restaurar sólo foco perdido por disabled, no robar foco de otro control.
- [CSS claro afecta editor/colecciones] → tokens acotados y regresión nativa completa en ambos temas; estilos no justifican retirar asserts de foco/errores.
- [Preferencia local no sobrevive borrar datos de WebView] → se documenta como preferencia de instalación, no dato autoritativo del workspace.

## Migration Plan

Sin migración ni cambios a SQLite v7, payloads, documentos, plantillas, vistas guardadas o manifiestos. No crear tablas vacías de Home/Goals/Weather. Backups existentes siguen restaurables; verificar original/restaurado consultando Home sin writes, incluidos v1/v2/v3 y formatos futuros.

Despliegue: contratos/operación de lectura y CLI → panel Home integrado → tokens/tema → tests/evidencia/documentación. Cliente viejo continúa leyendo v7 y sus workflows; no conoce home.query pero no recibe un workspace incompatible. Revertir UI/operación elimina Home sin pérdida de contenido; puede ignorar/borrar sólo la preferencia no secreta de tema, nunca workspace/originales. Registrar decisión de proyecciones/semánticas en ADR 0009 **al implementar**, con estado adoptado y evidencia.

Antes de sincronizar/archivar: core/CLI fmt/Clippy, contratos, tests de lifetime/tema, check/build/release y regresión nativa offline/reapertura/restauración; matriz visual y medición de lectura con corpus/latencia registrados. Objetivo orientativo p95 <300 ms por vista en el corpus de 10.000, sin inventar resultados: si no se alcanza, medir causa y resolver/documentar límite antes del cierre. No confundir validación OpenSpec con app ejecutada.

## Open Questions

Ninguna bloqueante para este tramo. Se registran para **cambios posteriores** el modelo/progreso Goals, proveedor/política de clima y modelo/autorización de fotos/organización; no se resuelven ni implementan desde este plan. Ajustes de espaciado/arte de portada se resuelven con capturas sintéticas respetando composición/contraste, sin pedir aprobación de cada detalle.
