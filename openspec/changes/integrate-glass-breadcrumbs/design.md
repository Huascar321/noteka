# Design

## Context

Home tiene una fila de breadcrumb antes del componente que contiene la portada. El canvas ASCII es transparente y dibuja sobre el fondo de Home, pero no puede aparecer detrás de la fila previa.

## Goals / Non-Goals

Hacer que el ASCII pase detrás de una cápsula de navegación, conservar el material neutro y la legibilidad en todas las secciones. No duplicar el canvas ni prometer la refracción nativa de Apple.

## Decisions

Colocar el breadcrumb y el contenido en la misma celda de grid, también en las vistas de sección: la navegación queda superpuesta en la parte superior con eventos limitados a sus controles y el contenido pasa por debajo al scrollear. Dar a las migas vidrio claro: tinte neutro de baja opacidad, borde luminoso, brillo superior, sombra de texto y filtro local pequeño solo cuando el WebView lo admite. La cápsula sobre ASCII usa un paso menos de opacidad que sobre la foto. El contenido conserva un inset superior para no quedar oculto en reposo. El elemento actual sigue siendo semánticamente `aria-current`, sin botón sin acción. Con menor transparencia o más contraste se usa fondo sólido y sin filtro. Conservar los tamaños de canvas y no animar el material.

En páginas de detalle, retirar también el fondo de la barra fija que contiene el breadcrumb. Agrupar las acciones de esa barra en una cápsula pequeña y translúcida, sin `backdrop-filter`: un filtro en ese ancestro cambia el bloque contenedor de los menús de posición fija y los desplaza. El cuerpo del editor permanece sólido. En el sidebar, sólo la sección activa hereda la cápsula; las demás filas no llevan fondo persistente y el hover usa `--hover`. La cápsula es un indicador único detrás de las filas y se desplaza con una transición elástica; con movimiento reducido el cambio es inmediato. En el breadcrumb, volver a una miga anterior colapsa las migas de la derecha hacia ella con una transición suave y hace un pequeño pop en la miga destino. Al entrar desde Home a una sección, la miga de nivel 2 nace desde Home y se desplaza hasta su posición con la misma transición. Las tabs de vista comparten una acción que mide la tab activa y desplaza/redimensiona un único indicador gris con el brillo y la sombra del vidrio, con una curva elástica más contenida. El indicador se monta en el toolbar, fuera del contenedor con scroll, para no recortar la sombra; se reposiciona al scrollear. La misma acción dispara el Jelly del sidebar sobre el icono clickeado, envolviendo el SVG en un grupo `.fx` para conservar su figura.

## Risks / Trade-offs

El contenido de Home asciende la altura de la fila eliminada, por lo que se verifica scroll y posición de controles. El filtro CSS suaviza el contenido bajo las cápsulas, pero no implementa desplazamiento refractivo nativo.
