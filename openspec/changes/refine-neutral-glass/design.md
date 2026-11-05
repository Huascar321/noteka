# Design

## Context

Noteka tiene una ventana opaca. Las referencias muestran vidrio que toma el color del contenido situado detrás, con un borde luminoso y controles neutros; no hay un fondo de escritorio disponible detrás del sidebar.

## Goals / Non-Goals

Acercar el material al carácter sobrio de las referencias sin teñir los fondos con el color primario. Mantener el contenido legible y el costo de composición acotado. No replicar refracción nativa ni hacer transparente la ventana del sistema.

## Decisions

Usar fondos y gradientes neutros estáticos para Home, sidebar y barra superior. Mantener el fondo compartido bajo la portada ASCII. La búsqueda se compacta dentro de los límites de la ventana. La búsqueda y los menús flotantes usan gris translúcido y filtro limitado a su propia superficie cuando el WebView lo permite; resaltar el borde superior con sombra interior. Las preferencias de menor transparencia o mayor contraste reciben superficies sólidas. El acento queda en foco, iconos y acciones.

## Risks / Trade-offs

Un fondo homogéneo revela menos la transparencia que una foto. El filtro local sigue teniendo costo gráfico; no se presume paridad con Liquid Glass de macOS ni rendimiento medido sin benchmark.

## Propagación

Compartir la receta neutra con menús de Tasks, tooltips visibles y diálogos pequeños. Aplicar solo gradientes y bordes al breadcrumb, barra de edición, barra de formato abierta, panel lateral de Settings y marco de side peek; sus cuerpos de lectura y edición conservan fondo sólido. Documentar tokens, límites, estados, temas, accesibilidad y verificación en `docs/design-system.md`. `AGENTS.md` la declara lectura obligatoria antes de crear o modificar pantallas.
