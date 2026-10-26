# Design

## Context

Ver propuesta. App mantiene panes por clave estable y WorkspacePane protege sus buffers. Los dos paneles tienen anchos CSS fijos y sólo el izquierdo anima. La preferencia existente de modo permanece independiente de dimensiones. Rust/SQLite siguen siendo autoridad de contenido.

## Goals / Non-Goals

**Goals:** límites estrictos en coordenadas del viewport, interacción continua y exclusión reversible sin perder ediciones.

**Non-Goals:** paneles anidados, guardar sesiones, mover contenido al cliente, nuevas operaciones de negocio o capturas de la pantalla del usuario.

## Decisions

- Módulo puro de preferencias/límites: `noteka.panel-widths.v1`, píxeles preferidos globales y anchos efectivos acotados al viewport. Default izquierdo 250px, Side peek 45% si aún no se eligió un ancho. Mínimos deseados 180/280px se reducen cuando el máximo porcentual es menor. Resize de ventana no sobrescribe la preferencia.
- Borde compartido con pointer capture, actualización agrupada por RAF, finalización que persiste una sola vez, cancelación que recupera snapshot y ajuste por teclado. Durante el arrastre se desactivan las transiciones; no se llama al núcleo.
- Estado de visibilidad Side separado de existencia del pane. Ocultarlo mantiene editor, buffer, revisión, selección, historial y scroll; inert/aria-hidden/atajos se actualizan al inicio. La misma identidad seleccionada vuelve a revelar sin recargar el original. Otra identidad usa la guardia del pane existente.
- Al abrir/revelar o ampliar un Side mayor al 35%, se contrae el lateral. Al abrir el lateral se suprime el Side ancho. No se abre el lateral automáticamente al achicar Side. Activación de pestaña y resize reaplican la regla. Una generación de apertura evita reaperturas tardías si la acción se invierte.
- Ancho constante del panel y desplazamiento de su borde derecho/opacidad de 260ms con el easing del lateral; el origen anima su espacio. Se evita transform permanente porque convertiría el pane en containing block de sus menús fixed. Cierre explícito espera la contracción tras resolver la guardia. Movimiento reducido hace la operación inmediata; no se desmonta por ocultar. Preferencias y dimensiones pertenecen al cliente, sin migración SQLite.
- Indicador: rail 24px, marcas 18/16/14/12px, activa +2px, paso 14px y grosor 2px, sin brillo. Se conservan objetivos de 24px para puntero grueso.

## Risks / Trade-offs

- [Buffers ocultos inaccesibles al cerrar] → activar dueño y revelar antes de preguntar; los peeks ocultos participan en setPending y cierre agregado.
- [Cambios rápidos o captura perdida] → generaciones de apertura, cancelación de RAF/capture y snapshot recuperable.
- [Ventana estrecha] → caps porcentuales prevalecen sobre mínimos; cabecera/campos adaptables y scroll local, con revisión a 420px.
- [Atajos o refresh sobre un pane oculto] → foco, visibilidad y activación de pestaña continúan separados; ocultar no consulta proyecciones.
- [Persistencia no disponible] → elección de sesión operativa y aviso recuperable, sin afectar el workspace.

## Migration Plan

Preferencia local aditiva; valores ajenos o corruptos usan defaults. Revertir el cliente ignora esa clave y deja SQLite/documentos intactos. Verificación sintética/headless, sin pantalla nativa ni workspace personal.
