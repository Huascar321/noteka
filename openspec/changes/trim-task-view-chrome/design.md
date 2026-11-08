# Design

## Context

`HomeTaskSection` comparte las vistas de Tasks entre la página Tasks y Home.
`HomeReader` resuelve el tamaño de página por vista y acumula páginas con
cursor; `TaskCalendar` dibuja Week y Month a partir de `page.items`.

## Decisions

- El tamaño de página vive donde se construye cada lector
  (`TasksPage.svelte` y `HomePanel.svelte`): Week/Month mantienen 100 y
  Scheduled/No Due/All usan 14. Ambos lectores aplican el mismo mapa para que
  Home y la página Tasks no diverjan.
- Week y Month dejan de ofrecer Load more y dejan de dibujar el bloque `No
  date`; la consulta del núcleo sigue incluyendo esas tareas, pero la interfaz
  no las presenta. La página del período queda acotada a 100 tareas.
- La nota explicativa de Inbox se retira del componente compartido: no hay
  variante que la muestre.
- El aviso de páginas reiniciadas se elimina del estado de lectura Home
  (`notice`), del componente de estado y de sus pruebas. La actualización
  conserva la regla de volver a la primera página y deja el cursor anterior
  disponible para volver a cargar las demás.

## Risks / Trade-offs

Si un período supera cien tareas, las restantes no son alcanzables desde la
interfaz; el calendario no declara esa cobertura. Las tareas sin fecha tampoco
son alcanzables desde Week/Month, aunque siguen disponibles en Inbox, No Due y
All. El retiro del aviso de páginas reiniciadas afecta a todas las secciones
Home, no sólo a Tasks.
