# Home: referencia visual confirmada

El 12 de octubre de 2026 el usuario aportó siete capturas y pidió copiar este Home. Esta referencia amplía la descripción visual del plan; no significa que el Home ni todas las funciones visibles estén implementados. Las capturas contienen datos personales: no se incorporan nombres, notas, fotos de contactos ni credenciales a fixtures o Git.

También autorizó inspeccionar su workspace Ultimate Brain con Playwright CLI y su perfil persistente para afinar la referencia 1:1, en modo lectura. El primer intento mostró login; después de que el usuario inició sesión manualmente el 12/10, se verificó acceso al Home y se inspeccionaron sus seis secciones. Esta autorización no permite modificar Notion, importar notas ni descargar fotos personales como assets. Snapshots y capturas privados quedan fuera del checkout; acá se conserva sólo la descripción genérica de estructura/comportamiento.

## Composición que hay que reproducir

- Página Home de ancho amplio, portada panorámica azul/verde, icono de casa superpuesto, título grande y navegación compacta desplegable: Tasks, Notes, Projects, Tags y Content.
- Franja meteorológica: localidad configurable, condición/temperatura actual y siete días. Debe ser opt-in, con proveedor visible y estado sin red/caché; no geolocalización ni envío de notas. No copiar números del screenshot como clima real.
- Secciones verticales en orden: **Tasks → Notes → Projects → Tags → Goals → People**. Cabeceras con icono, título/enlace, fondo azul suave y creación contextual cuando corresponde.
- Cada sección tiene su propia barra de vistas/pestañas y controles compactos; conserva las entidades originales, no copias por pestaña. Presentar sólo controles funcionales, no botones decorativos que aparenten capacidades pendientes.
- Tasks: creación rápida, listas compactas agrupadas por proyecto, grupos plegables, cantidad y captura en grupo; a la derecha relaciones/fecha. Referencia de pestañas: Today, Inbox, Week, Month, Scheduled, No Due, Recurring, Active Projects, Active Projects (In Progress), Active Content, All y Done (en el menú de vistas adicionales).
- Notes: filas compactas con icono/título a izquierda y relaciones, clasificación y antigüedad a derecha; paginación/carga adicional. Pestañas de referencia: Notes, Inbox, Fav., Clips, Voice, Journal, Meetings y All.
- Projects: vistas Active/Planned/Board. Board con columnas Planned/Doing/Ongoing/Done/On Hold, colores suaves, tarjetas con título, progreso, pendientes/vencidas, contexto y acción New page por grupo. El progreso/contador debe provenir de reglas documentadas, no valores ficticios.
- Tags: listas agrupadas por tipo Area/Resource/Entity, encabezados plegables con cantidad; última nota/referencia y contador de notas a derecha. Pestañas Recent/Fav./A–Z/Types.
- Goals: grupos Active/Dream/Achieved, relaciones/contexto, progreso/fecha y vistas By Activity/By Deadline/Achieved. Objetivos pasan a ser alcance solicitado del Home; su modelo y fórmula de progreso necesitan un cambio propio.
- People: galería con foto opcional, nombre, etiquetas múltiples y organización; tarjetas de ancho consistente y creación/paginación. Pestañas Contacts/New/Updated/Check-Ins/Birthdays/Pipeline/Active Projects.

## Tema y límites

Las capturas son claras; se reproducirá estructura/jerarquía y se mantendrá la preferencia previa de tema oscuro como opción, sin tratar el color claro como requisito de cambiar datos o dominio. Portada/fotos deben usar originales locales autorizados; no descargar fotos de personas ni reutilizar una sesión de Notion para obtenerlas.

No se copian los controles de colaboración, Share/Private ni branding de Notion como si Noteka tuviera esas funciones. Content, recurrencias, favoritos, voz, check-ins, cumpleaños, pipeline y clima figuran en las capturas, pero no se declaran implementados por mostrarlas. Cada capacidad pendiente requiere comportamiento y pruebas antes de habilitarse. No inventar módulos fuera de la referencia ni importar contenido personal automáticamente.

## Inspección del Home autenticado — 12/10/2026

Se recorrió la página por scroll, se leyeron las pestañas y se abrieron sólo los controles de lectura de filtros/vistas adicionales. No se crearon páginas ni se editaron propiedades, fórmulas, filtros, relaciones o registros. No se abrieron notas privadas ni páginas de credenciales. Las capturas del Home contienen títulos/fotos personales visibles y **no** se incorporan a Git ni se usan como fixtures/assets.

| Sección | Vista seleccionada al inspeccionar | Presentación observada |
| --- | --- | --- |
| Tasks | Today | Lista sin grupos en esta vista, indicador de estado a izquierda; relación a proyecto y vencimiento a derecha. Las capturas iniciales también muestran listas agrupadas: son presentaciones distintas, no hay que forzar agrupación en todas. |
| Notes | Notes | Filas compactas con icono; relaciones, clasificación y antigüedad alineadas a derecha; New page y Load more debajo. |
| Projects | Active | Lista agrupada por estado, con grupos Doing/Ongoing y cantidades; relaciones y resumen de tareas activas/vencidas a derecha. Board sigue confirmado por las capturas iniciales, no por un cambio de pestaña durante esta inspección. |
| Tags | Recent | Lista sin grupos con referencia a nota y etiqueta de tipo a derecha. Types agrupada sigue confirmada por las capturas iniciales. |
| Goals | By Activity | Grupos Active/Dream/Achieved, incluidos vacíos; contexto, porcentaje/barra y fecha a derecha. El campo exacto de esa fecha y la fórmula del progreso no se deducen de su apariencia. |
| People | Contacts | Galería con foto/espacio vacío, nombre en varias líneas, etiquetas de relación y organización; tarjetas uniformes y acción New page. |

El filtro **Today** se leyó directamente: `Status is not Done AND Due (End date) is on or before Today`. Incluye vencidas; no equivale a "vence exactamente hoy" ni demuestra equivalencia con My Day de Noteka. Esa semántica se especificará en el cambio Home sin alterar silenciosamente los filtros de productividad existentes. También se comprobó el menú de vistas: Done es la vista adicional de Tasks. No se inspeccionaron reglas de las demás pestañas ni se infirió su lógica a partir del nombre.

Referencia de composición en viewport de 1920 × 845: portada panorámica a todo el ancho, contenido con márgenes amplios, cabeceras azules a todo el ancho útil, barra compacta de pestañas con selección en píldora, filas de una línea y separación generosa entre secciones. El clima visible atribuye su widget a Indify; observarlo no adopta ese proveedor ni verifica frescura de la previsión. No se probó layout móvil. La ventana se dejó abierta en el Home, sin diálogos de inspección.

## Secuencia de implementación

1. `add-advanced-collection-views` está verificado y archivado. Su galería textual inicial todavía no cumple People con fotos.
2. [Templates/bases incrustadas](verification-phase-3c-templates.md) archivado (17/17). [`add-local-home-dashboard`](../openspec/changes/add-local-home-dashboard/proposal.md) está implementado, con **9/17 tareas plenamente verificadas**, todavía sin archivar. Ofrece composición y quince vistas locales sin modificar workflows previos. [Guía/cobertura real](home.md) y [evidencia](verification-home.md): Goals/clima pendientes, galería textual sin fotos, auxiliares sin controles falsos. **606 checks headless aprobados** por pedido del usuario; [revisión final cerrada](review-home.md), aceptación nativa integrada pendiente. No cambia la referencia final ni equivale a una copia completa.
3. Añadir Goals y funciones auxiliares en cambios independientes cuando sus dependencias estén disponibles. Clima remoto y originales/fotos tienen políticas explícitas; core offline y ausencia de LLM se conservan.

La fidelidad se verificará con capturas del Home ejecutado y recorridos reales de creación, cambio de vista y apertura del mismo ID. La apariencia sola no demuestra comportamiento ni paridad completa con Notion/Ultimate Brain.
