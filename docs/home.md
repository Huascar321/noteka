# Home local

Home es una proyección de originales, no una entidad/documento ni colecciones espejo. El núcleo publica `home.query` y la CLI `home query`; la interfaz no calcula filtros o resúmenes. SQLite sigue en **v7**, documentos v1/v2/v3 y formatos futuros se conservan. No usa LLM, Internet, fotos remotas ni modelos descargados.

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
| tasks_today / Today | todo, due_date <= día | due_date asc, created_at asc | Lista |
| tasks_inbox / Inbox | todo, inbox true | created_at asc | Grupos por proyecto |
| tasks_all / All | Todas | created_at asc | Grupos por proyecto |
| tasks_done / Done | done | updated_at desc | Grupos; menú Más vistas |
| notes_all / Notes | Todas | updated_at desc | Lista |
| notes_inbox / Inbox | inbox true | created_at asc | Lista |
| projects_active / Active | doing/ongoing | doing→ongoing, título | Grupos |
| projects_planned / Planned | planned | título | Lista |
| projects_board / Board | Todas | planned→doing→ongoing→done→on_hold, título | Cinco columnas |
| tags_recent / Recent | Todas | updated_at del tag desc | Lista |
| tags_alphabetical / A–Z | Todas | título | Lista |
| tags_types / Types | Todas | area→resource→topic, título | Grupos |
| people_contacts / Contacts | Todas | título | Galería textual |
| people_new / New | Todas | created_at desc | Galería textual |
| people_updated / Updated | Todas | updated_at desc | Galería textual |

Today incluye vencidas, excluye futuras/sin vencimiento/done. **My Day sigue planned_date == día; Overdue sigue due_date < día**. Home Active es doing/ongoing; el workflow lateral Active Projects mantiene planned/doing/ongoing. **Topic no se renombra Entity**. Recent no ordena por última nota. New/Updated no son filtros de período. Sólo Today se contrastó con filtro de referencia; el resto son reglas propias de Noteka, no lógica Notion inferida.

Grupos referencian IDs de una página de elementos únicos; una tarea con dos proyectos aparece dos veces con el mismo ID. Cantidad del grupo = apariciones de página, no total del workspace. Se incluyen Sin proyecto, dos estados Active, cinco Board y tres Types vacíos. Un grupo vacío puede tener registros en otra página.

## Metadatos y resúmenes

Lectura conservadora por claves conocidas: campos extras permanecen intactos; estado/default válido sólo por omisión legítima en objeto heredado (Task todo/normal/inbox true, Project planned, Note inbox true). Un null en estado/Inbox no es default; fechas opcionales null sí son ausencia. Payload no objeto y valores presentes incompatibles son no disponibles. All/Board/Types pueden conservar identidad con aviso/grupo No disponible; vistas filtradas indeterminadas no cuentan como match y exponen `unavailable_count`. Un campo irrelevante no invalida el resto de la fila.

- Proyecto: todas sus tareas únicas activas directamente relacionadas, no sólo las 50 filas visibles. `pending`, `done`, `overdue` (todo y due_date < día), `total`; progreso entero comprobado `floor(100 * done / total)`. Total cero → porcentaje null/**Sin tareas**. Metadatos requeridos incompatibles → resumen no disponible, no cero. No persiste porcentaje ni cambia estado del proyecto. No es una fórmula atribuida a Notion o Goals.
- Tag: todas sus notas únicas activas, conteo y última por updated_at desc/ID asc. Sin notas → 0/last null. Timestamp no interpretable → resumen no disponible; Inbox/payload no afectan identidad y fecha de la última nota. No carga documentos.
- Nota: antigüedad civil firmada desde timestamp y zona/día de consulta en Rust; fecha futura e inválida se distinguen, no se clampa a cero.

## Operación y navegación

Abrir/crear workspace entra en Home con sidebar y ancho restante, sin entity-list. Navegación desplegable lleva a pantallas reales. Cabeceras/+ New page comienzan buffers del editor existente; columnas Board pasan sólo estado, Types clasificación, grupo Project contexto original. **Today no asigna due_date/planned_date**. No hay writes antes de Guardar. Apertura/captura protegen también la lectura inicial: navegación o atajos competidores no crean otro buffer mientras se prepara el original; la captura libera el bloqueo antes de enfocar su título.

Guardar mantiene abierto el original; **Volver a Home** protege guardar/descartar/cancelar también para buffers de colección. Cancelar/fallar/conflicto no monta Home sobre el buffer. Guardar y continuar confirma una creación una sola vez: un fallo posterior en Home ofrece reintentar lectura, no repetir creación. Completar/reclasificar se hace en el editor con revisión esperada, no mediante checkbox decorativo en Home.

Tabs son botones `aria-pressed` con Tab/Enter/Espacio del navegador; grupos usan details/summary plegables. Siguiente reemplaza página; Inicio la reinicia, no acumula el workspace. Actualizar por sección o Home y reactivación de ventana leen originales sin polling. Cola Home máximo **dos** IPC; generación/lifetime descartan resultados antiguos, cancelan encoladas y conservan cupo de IPC iniciado hasta resolver. Fallo de una sección limpia su proyección, no las demás. Al desmontar/cambiar workspace se desechan los lectores.

## Cobertura y tema

Portada CSS propia azul/verde, casa, navegación y orden **Tasks → Notes → Projects → Tags → Goals → People**. Cabeceras, filas/grupos, cinco columnas y galería textual funcionan con datos originales. Franja clima y Goals permanecen en posición con avisos, sin datos ficticios. No es paridad completa con Ultimate Brain.

| Zona | Implementado | Pendiente |
| --- | --- | --- |
| Portada/nav | Jerarquía, enlaces locales, dos temas | Personalizar portada; Content |
| Clima | Franja/aviso sin llamadas | Localidad/proveedor, opt-in/caché, siete días |
| Tasks | Today/Inbox/All/Done | Week/Month/Scheduled/No Due/Recurring, Active Projects/(In Progress)/Active Content |
| Notes | Notes/Inbox | Fav./Clips/Voice/Journal/Meetings/All diferenciada |
| Projects | Active/Planned/Board y resumen Noteka | Paridad con campos/fórmulas de referencia no inspeccionados |
| Tags | Recent/A–Z/Types; Topic conservado | Fav.; tipo Entity requiere decisión/modelo propio |
| Goals | Cabecera/aviso | Entidad, Active/Dream/Achieved, relaciones/fechas/progreso y By Activity/By Deadline/Achieved |
| People | Contacts/New/Updated textual, relaciones directas | Fotos/organización, Check-Ins/Birthdays/Pipeline/Active Projects |

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
