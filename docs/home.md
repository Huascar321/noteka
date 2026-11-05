# Home local

> Estado 28/10/2026: el workspace usa esquema SQLite v20. Los números v7 y
> límites históricos de las secciones antiguas de esta guía describen el cierre
> inicial de Home; ver el plan general y la [verificación actual](verification-focus-refresh.md).

Home es una proyección de originales, no una entidad/documento ni colecciones espejo. El núcleo publica `home.query` y la CLI `home query`; la interfaz no calcula filtros o resúmenes. El workspace usa esquema SQLite **v20**, y los documentos v1/v2/v3 y formatos futuros se conservan. Las vistas locales no usan LLM, Internet, fotos remotas ni modelos descargados. El clima es una función externa optativa independiente, descrita abajo.

## Clima optativo

Set location solicita directamente la ubicación actual en macOS y pide permiso al sistema. Apple puede recibir coordenadas para identificar la localidad y Open-Meteo las recibe para el pronóstico. No hay selección manual; en Windows y Linux se informa que la ubicación automática todavía no está implementada. La ubicación activada y el último pronóstico se guardan en localStorage de la aplicación, no en SQLite ni backups del workspace. Las ciudades fijas guardadas anteriormente se ignoran hasta activar la ubicación actual. Disable elimina los datos guardados. Home muestra condiciones actuales, hoy y los siete días siguientes en la zona horaria local; vuelve a consultar la ubicación al abrir y cada hora. Si falla la red o el permiso, conserva el último resultado e indica su hora y un error recuperable. Los íconos locales representan los códigos meteorológicos con movimiento sutil; la condición actual distingue día y noche cuando el proveedor lo indica. Las animaciones se pausan al ocultar Home o la ventana y respetan movimiento reducido. La atribución visible se presenta como enlace Open-Meteo. El servicio alojado gratuito de Open-Meteo es para uso no comercial y requiere atribución; su disponibilidad no está garantizada.

## Contrato y límites

Input estricto: `{ "view": "tasks_today", "as_of": "2026-10-12", "limit": 50, "cursor": null }`. Sólo `view` es obligatorio. Día civil canónico 0001–9999; sin día/cursor se resuelve en la zona IANA del workspace, no la del sistema. Límite 1–100, por defecto/UI 50.

Salida `HomePage`: `view`, `date`, `timezone`, `items`, `groups`, `matching_count`, `unavailable_count`, `offset`, `next_cursor`. Cada item tiene `entity` ligera (ID/tipo/título/revisión/timestamps/archivo), `metadata` conocida con `issues`, `context` de referencias, `context_total` y `summary` de proyecto/tag. **No contiene documento, payload completo, adjuntos ni texto de notas**. Abrir usa `entity.get` original vigente; una versión futura no se convierte para mostrar Home.

Cursor: string JSON v1, scope ID de workspace + SHA-256 de ruta canónica, vista/día/offset/límite; máximo 2 KiB. Con cursor sin `as_of` mantiene el día de esa página aunque cruce medianoche. Rechaza versión, campos, scope, vista, día, offset o límite incompatibles. No es firmado ni autorización; previene mezcla accidental. Otra ruta restaurada no acepta el cursor previo.

Caps por llamada: **10.000 candidatos activos del tipo**, **10.000 fuentes distintas de resumen**, **50.000 vínculos incidentes a la página**, **200 grupos**. Un exceso devuelve `validation_error` con `details.limit/maximum`, nunca resumen parcial como completo. Contexto: 50 chips por fila, orden tipo/título/ID, con `context_total` y aviso +N; los grupos sí usan todos los proyectos relacionados. Los caps no restringen el CRUD o la búsqueda anteriores.

Cada llamada adquiere un snapshot SQLite de lectura Deferred antes de selección/contexto/resúmenes. No escribe dominio, revisiones, relaciones, FTS ni historial. Cada sección de Home tiene snapshot independiente; **todo Home no es una transacción única**. Sin escrituras concurrentes las páginas recorren el mismo orden. Con escrituras, Actualizar/Inicio reinician la página; no se promete una colección congelada entre llamadas.

## Quince vistas y reglas

Todas excluyen principales archivadas y conservan contexto archivado. Todo orden termina en ID ascendente. Título normalizado significa `to_lowercase` Unicode de Rust, no collation española/Notion.

| Vista / UI | Selección | Orden previo al ID | Presentación |
| --- | --- | --- | --- |
| tasks_today / Today (núcleo) | todo/doing, due_date <= día | due_date asc, created_at asc | Lista |
| tasks_inbox / Inbox | todo/doing sin relación Project local | created_at asc | Grupos por proyecto |
| tasks_all / consulta histórica | Todas | created_at asc | Grupos por proyecto |
| tasks_pending / All | todo/doing | created_at asc | Grupos por proyecto |
| tasks_calendar_week / Week | Semana de Due, sin fecha y fecha inválida; todos los estados | fecha, sin fecha al final | Calendario semanal; páginas cargadas |
| tasks_calendar_month / Month | Mes de Due, sin fecha y fecha inválida; todos los estados | fecha, sin fecha al final | Calendario mensual; páginas cargadas |
| tasks_done / Done | done | updated_at desc | Grupos; menú Más vistas |
| notes_all / Notes | Todas | updated_at desc | Lista |
| notes_inbox / Inbox | notas activas sin relación Project local | created_at asc | Lista |
| projects_active / Active | doing/ongoing | doing→ongoing, título | Grupos |
| projects_planned / Planned | planned | título | Lista |
| projects_board / Board | Todas | planned→doing→ongoing→done→on_hold, título | Cinco columnas |
| projects_calendar / Deadlines | No archivados, incluidos Done y sin fecha | fecha de fin, sin fecha al final | Calendario mensual; páginas cargadas |
| tags_recent / Recent | Tags admisibles | actividad de origen o notas relacionadas desc | Lista |
| tags_alphabetical / A–Z | Todas | título | Lista |
| tags_types / Types | Tags admisibles | area→resource→entity→topic, título | Grupos |
| people_contacts / Contacts | Contactos admisibles | orden de galería, título | Galería |
| people_new / New | Contactos creados desde el último mes civil | orden de galería, título | Galería |
| people_updated / Updated | Contactos actualizados desde el último mes civil | orden de galería, título | Galería |

Today incluye vencidas, excluye futuras/sin vencimiento/done. **My Day sigue planned_date == día; Overdue sigue due_date < día**. `doing` cuenta como pendiente en las consultas históricas Today, Inbox, Week, Month, Scheduled, No Due, proyectos activos y resúmenes; `done` es el único estado completado. Los nuevos calendarios visuales Week/Month incluyen Done. Home Active es doing/ongoing; el workflow lateral Active Projects mantiene planned/doing/ongoing. La tab visual **Today** de Tasks usa `tasks_projects_in_progress` (pendientes de proyectos Doing/Ongoing, agrupadas); el núcleo conserva `tasks_today` por vencimiento para CLI/MCP. **Topic no se renombra Entity** y su grupo aparece cuando contiene registros. Tags Recent combina actividad de origen y notas relacionadas. People New/Updated usa fechas de origen cuando existen y fechas locales como fallback; el límite inferior es un mes civil inclusivo. El orden acreditado de galería precede al título y al ID. Son reglas locales, no lógica Notion inferida.

Grupos referencian IDs de una página de elementos únicos; una tarea con dos proyectos aparece dos veces con el mismo ID. Cantidad del grupo = apariciones de página, no total del workspace. Se incluyen Sin proyecto, dos estados Active, cinco Board y tres Types vacíos. Un grupo vacío puede tener registros en otra página.

## Metadatos y resúmenes

Lectura conservadora por claves conocidas: campos extras permanecen intactos; estado/default válido sólo por omisión legítima en objeto heredado (Task todo/normal, Project planned, Note). La bandera `inbox` conserva su default `true` al crear, pero ninguna vista la consulta: Inbox se deriva de la ausencia de relación Project ([ADR0059](decisions/0059-derived-inbox.md)). Un null en estado no es default; un null en inbox queda como dato heredado inerte; fechas opcionales null sí son ausencia. Payload no objeto y valores presentes incompatibles son no disponibles. All/Board/Types pueden conservar identidad con aviso/grupo No disponible; las filas cuya metadata completa no puede interpretarse se excluyen de esa vista y exponen `unavailable_count`, conservando el original recuperable. Un campo irrelevante no invalida el resto de la fila.

- Proyecto: todas sus tareas únicas activas directamente relacionadas, no sólo las 50 filas visibles. `pending` (todo/doing), `done`, `overdue` (todo/doing y due_date < día), `total`; progreso entero comprobado `floor(100 * done / total)`. Total cero → porcentaje null/**Sin tareas**. Metadatos requeridos incompatibles → resumen no disponible, no cero. No persiste porcentaje ni cambia estado del proyecto. No es una fórmula atribuida a Notion o Goals.
- Tag: todas sus notas únicas activas, conteo y última por updated_at desc/ID asc. Sin notas → 0/last null. Timestamp no interpretable → resumen no disponible; la bandera heredada inbox y el payload no afectan identidad y fecha de la última nota. No carga documentos.
- Nota: antigüedad civil firmada desde timestamp y zona/día de consulta en Rust; fecha futura e inválida se distinguen, no se clampa a cero.

## Operación y navegación

Abrir/crear workspace entra en Home con sidebar y ancho restante, sin entity-list. Navegación desplegable lleva a pantallas reales. Cabeceras/+ New page comienzan buffers del editor existente; columnas Board pasan sólo estado, Types clasificación, grupo Project contexto original. **Today no asigna due_date/planned_date**. No hay writes antes de Guardar. Apertura/captura protegen también la lectura inicial: navegación o atajos competidores no crean otro buffer mientras se prepara el original; la captura libera el bloqueo antes de enfocar su título.

Guardar mantiene abierto el original; **Volver a Home** protege guardar/descartar/cancelar también para buffers de colección. Cancelar/fallar/conflicto no monta Home sobre el buffer. Guardar y continuar confirma una creación una sola vez: un fallo posterior en Home ofrece reintentar lectura, no repetir creación. Completar/reclasificar se hace en el editor con revisión esperada, no mediante checkbox decorativo en Home.

Tabs son botones `aria-pressed` con Tab/Enter/Espacio del navegador; grupos usan details/summary plegables. Load more acumula páginas; una actualización correcta vuelve a la primera y avisa si había páginas adicionales. Actualizar por sección o Home y reactivar la ventana leen originales sin polling ni detector persistente. Las lecturas coordinadas comparten máximo **dos** IPC por workspace; generaciones descartan respuestas antiguas y cancelan encoladas sin liberar anticipadamente el cupo de una operación iniciada. Durante el refresco se conservan los resultados de la misma consulta; si falla, permanecen identificados como anteriores, con reintento y sin usar su cursor. Un cambio de vista, día o workspace retira la identidad anterior. El tablero no permite mover proyectos con datos pendientes o no verificados. Los borradores y el historial del editor se conservan, y una revisión externa con cambios locales produce conflicto.

## Cobertura y tema

Portada CSS propia azul/verde, casa, navegación y orden **Tasks → Notes → Projects → Tags → Goals → People**. Cabeceras, filas/grupos, cinco columnas y galería de personas con fotos locales opcionales funcionan con datos originales. [Goals](goals.md) muestra sus originales y progreso por hitos; la franja de clima conserva el aviso pendiente. No es paridad completa con Ultimate Brain.

| Zona | Implementado | Pendiente |
| --- | --- | --- |
| Portada/nav | Jerarquía, enlaces locales, dos temas | Personalizar portada; Content |
| Clima | Franja/aviso sin llamadas | Localidad/proveedor, opt-in/caché, siete días |
| Tasks | Today (pendientes de proyectos Doing/Ongoing)/Inbox/All/Done | Week/Month/Scheduled/No Due/Recurring, All Active Projects/Active Content |
| Notes | Notes/Inbox | Fav./Clips/Voice/Journal/Meetings/All diferenciada; Unclassified (biblioteca sin Tag/Project/People) |
| Projects | Active/Planned/Board y resumen Noteka | Paridad con campos/fórmulas de referencia no inspeccionados |
| Tags | Recent/A–Z/Types; Topic conservado | Fav.; tipo Entity requiere decisión/modelo propio |
| Goals | Páginas tipadas, Active/Dream/Achieved, relaciones/fechas/progreso por hitos y By Activity/By Deadline/By Tag/Achieved | Sincronización o fórmula universal de actividad |
| People | Contacts con fotos locales/relaciones/empresa, New/Updated/Check-Ins, calendario Birthdays, Pipeline y Active Projects por persona | Sincronización continua, recordatorios y arrastre de Pipeline |

Tema en `localStorage` WebView, clave **noteka.theme.v1**, sólo dark/light. Oscuro inicial/fallback ante valor inválido o lectura inaccesible. Fallo de escritura permite tema activo y avisa que no se guardó. No guarda notas/rutas/secretos, no toca SQLite ni buffers y no pertenece al backup del workspace. Borrar datos WebView elimina la preferencia, no el contenido. Scroll/vistas/grupos son efímeros por ruta.

Objetivos de contraste normal 4,5:1 y foco/controles 3:1, tokens compartidos para editor/colecciones/Home. Pruebas UI predeterminadas: `npm run test:headless`, sin ocupar pantalla; aceptación del host nativo pendiente y sujeta a consentimiento específico. No es certificación WCAG ni prueba de lectores de pantalla/teclado físico. Evidencia/layouts/latencia: [verification-home.md](verification-home.md); [revisión final y trazabilidad de escenarios](review-home.md).

## CLI reproducible

Crear sólo un directorio nuevo fuera del checkout:

```sh
export W="$(mktemp -d "${NOTEKA_TEST_TMP:-${TMPDIR%/}}/noteka-home-demo.XXXXXX")/workspace"
printf '{"timezone":"America/La_Paz"}' | cargo run --offline -q -p noteka-cli -- --json --workspace "$W" workspace create
printf '{"kind":"task","title":"Tarea sintética Home","payload":{"due_date":"2026-10-12"}}' | cargo run --offline -q -p noteka-cli -- --json --workspace "$W" entity create
printf '{"view":"tasks_today","as_of":"2026-10-12"}' | cargo run --offline -q -p noteka-cli -- --json --workspace "$W" home query
printf '{"view":"projects_board","as_of":"2026-10-12","limit":50}' | cargo run --offline -q -p noteka-cli -- --json --workspace "$W" home query
```

En stdout sólo Response JSON; errores/códigos como las demás operaciones. Esquema y TypeScript se generan desde Rust. [ADR 0009](decisions/0009-shared-home-projections.md) explica la decisión.

Actualización People del 24/10/2026: [guía de las siete vistas](people-reference.md) y [revisión acotada](verification-people-reference.md). La galería carga diez contactos y Load more acumula páginas; los miembros de otra colección People se conservan en el listado general.

Actualización del encabezado del 24/10/2026: se retiraron Navegación y Actualizar
Home. **⌘R / Ctrl+R o F5** refresca las lecturas del Home activo, conservando sus
vistas y los buffers de la sesión. [Revisión](verification-home-refresh.md).


## Pies y filas de acción al 24/10/2026

Los contadores y fechas de consulta no se muestran al pie de Tasks/Notes/
Projects/Tags/People; su cobertura queda en Opciones de vista. El aviso técnico
final de Home se retiró. Load more y New page sin borde ocupan toda la fila;
la captura con borde conserva su estilo. Cargar más acumula elementos únicos y
sólo se ofrece si existe otra página. Ver [revisión actual](verification-tasks-page.md).

## Orientación y detalles al 26/10/2026

El [indicador persistente](scroll-outline.md) identifica las seis secciones de
Home y sigue su scroll. Los originales y borradores abren según el
[modo de detalle](detail-peek.md), inicialmente Side peek derecho. El origen
conserva su vista y posición al mover el foco al detalle o cerrarlo; actualizar
explícitamente o reactivar la pestaña mantiene las reglas de consulta vigentes.
