# Design

## Context

La barra mide 44 px. La tira de pestañas agrega 6 px arriba; los botones de 32 px se centran en la fila. El lateral alterna display:none y cambia la cantidad de columnas, impidiendo interpolación. App ya separa sidebarCollapsed/sidebarOpen y aplica inert/aria-hidden sin desmontar WorkspacePane.

## Goals / Non-Goals

**Goals:** centros compartidos, transición reversible y contenido estable durante el movimiento.

**Non-Goals:** cambiar navegación, datos, APIs, preferencias persistentes o reiniciar la sesión nativa del usuario.

## Decisions

1. Pestañas de 32 px centradas como los botones, con borde completo y sin padding superior. Mantener la fila de 44 px.
2. Desplazar 8 px hacia abajo la posición configurada de los controles Mac (y=16→24). En Tao/Wry instalados, y determina la altura/posición del contenedor nativo, no la coordenada directa del centro. La captura acredita el desfase actual; la posición final nativa requiere comprobarse en el host.
3. Mantener dos columnas desktop (250 px→0 px y contenido flexible) y trasladar el lateral de ancho fijo con la misma curva de 260 ms. Evita comprimir/refluir sus etiquetas. visibility se retrasa sólo al cerrar; inert/aria-hidden se aplican inmediatamente. No usar display:none ni desmontar paneles.
4. En ancho estrecho, lateral absoluto de 240 px con la misma transición en ambos sentidos. Fondo montado mientras el viewport sea estrecho, con opacidad/visibilidad y retiro de interacción al cerrar. Reutilizar la regla global de movimiento reducido, sin temporizadores JS.

## Risks / Trade-offs

- Geometría de AppKit → contrastar configuración con implementación instalada y declarar pendiente la aceptación nativa; no dibujar controles falsos.
- Interrupciones o cambio de ancho → CSS toma el valor interpolado actual; comprobar inversión rápida, breakpoint y preservación del panel/borrador.
- Layout durante animación → transición breve y acotada a la shell; no afirmar fluidez física sin medición nativa.

## Migration Plan

Sin migración. CSS se aplica por recarga; la posición nativa configurada se aplica al próximo inicio normal del host. Revertir presentación/configuración no altera SQLite ni adjuntos.
