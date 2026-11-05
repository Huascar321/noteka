## Context

El canal WebKit/Blink vertical mide 9 px y el tirador tiene 1 px transparente por lado. El usuario pide una separación mínima, conservando la anchura visible acordada previamente.

## Goals / Non-Goals

Separar visualmente el tirador del borde sin alterar su ancho de tinta, el canal horizontal ni el scroll nativo. No sustituir el scrollbar del sistema.

## Decisions

Ampliar sólo el canal vertical a 10 px y aplicar 2 px de borde transparente al lado derecho del tirador vertical; el izquierdo sigue en 1 px, conservando 7 px de tinta. Mantener 8 px horizontales, colores, pista transparente y reglas específicas de ocultación. En motores sin estos pseudo-elementos se conserva el fallback estándar.

## Risks / Trade-offs

Las barras superpuestas y el sistema pueden representar los márgenes de modo diferente; comprobar motores, temas y scroll sin atribuir aceptación nativa no observada.
