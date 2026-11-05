# Design

## Context

`HomePersonCard` comparte galería, lista y pipeline. Las relaciones tienen dos botones hermanos dentro de un wrapper. `NotePageProperties` separa Tag/URL/checkbox/select nativos. `WorkspacePane` mantiene bloqueo y refresco fuera de `.actions`; esta última aloja estado, Save y PageTools. Ver `proposal.md` para la motivación.

## Goals / Non-Goals

**Goals:** conservar identidad de controles/datos y reforzar jerarquía con el material aprobado, comenzando por tarjetas y relaciones.

**Non-Goals:** modificar fotos, contenido, etiquetas semánticas, operaciones Rust, guardias, persistencia o temas de otras páginas de detalle.

## Decisions

- Tarjetas sólidas con radio 16 px en galería/pipeline y 12 px en filas compactas. Reutilizar borde direccional/sombra neutra del vidrio con intensidad menor; reflejos de interacción estáticos. Evitar blur y elevación animada por tarjeta para preservar fotos, lectura y densidad. New page comparte radio pero conserva su jerarquía secundaria.
- Relaciones con borde/reflejo tenue y radio de cápsula; apertura y eliminación continúan en botones separados, con × circular integrado. Mantener nombres completos envolventes y foco visible. No introducir avatares nuevos ni operaciones para resolverlos: el icono existente identifica al destino.
- Tag agrupa destinos navegables y edición en un marco de campo, con chips internos planos. Type conserva el select HTML y desplegable nativo; `appearance: none` normaliza el marco y una flecha decorativa sin eventos reemplaza el adorno del sistema. WebKit ignoraba altura/radio/sombra de la cápsula con apariencia automática. URL no aparenta ser editable; Favorite conserva su checkbox semántico. Se descarta convertir todos los valores en botones decorativos.
- Agrupar bloqueo/refresco dentro de `.actions` sólo cuando `plainNotePage` es verdadero; otras páginas conservan su estructura. La cápsula tiene borde, reflejo y tinte estático sin blur en el ancestro: los menús fixed siguen anclados al viewport. Mantener nombres, handlers, bind de foco y estado Save/Retry.
- Tokens de marco/campo compartidos y fallbacks al final de CSS; no añadir animadores, dependencias ni bucles. Rust y SQLite conservan propiedad de reglas/datos. Es un cambio visual sin migración ni nuevas escrituras.
- Corrección tras revisión del usuario: el wrap del marco Tag hacía saltar la flecha bajo una sola etiqueta cuando la columna era estrecha. Separar la lista envolvente de destinos y reservar la flecha fuera de ella; truncar el nombre individual manteniendo su title. Un destino o vacío ocupa 32 px; sólo varias etiquetas pueden añadir filas. El bloqueo atenúa la tinta de Type/flechas, conservando opacidad y material del marco, porque Tag mantiene navegación disponible.

## Risks / Trade-offs

- [Recorte de foco por tarjeta con foto] → anillo exterior en tarjeta al enfocar su botón, con capas decorativas sin eventos.
- [Cabecera ancha o nombre largo] → comprobar side/center/full, estado pendiente y 420–600 px; conservar ajuste de fila y wraps.
- [Nota bloqueada] → comprobar apertura de relaciones y bloqueo de edición, así como desbloqueo temporal/re-lock y guardado explícito.
- [Contraste compuesto] → medir texto en campos/cápsulas y conservar el fallback sólido; no afirmar equivalencia óptica nativa o rendimiento GPU.

## Migration Plan

Cambios locales con build/HMR, sin publicación ni migración. El checkpoint anterior `affca8a` sigue disponible y se preservan los cambios locales ya aceptados; con commit local conjunto autorizado posteriormente, sin push, sync o archivo automático.
