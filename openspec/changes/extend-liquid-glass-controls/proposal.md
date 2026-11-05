# Proposal

## Why

El usuario aprobó el vidrio del sidebar y pidió extender su lenguaje a los cuatro componentes señalados: pestaña superior, breadcrumb, vista activa y acción New. Actualmente muestran materiales distintos.

## What Changes

- Pestaña superior activa como cápsula neutra de vidrio, con sombra corta y controles +/× circulares al interactuar.
- Breadcrumb con reflejo y contorno más tenues; vista activa con el mismo vidrio neutro y menor relieve que el sidebar.
- New con vidrio teñido por el color primario y texto blanco.
- Compartir la óptica del sidebar, mantener dimensiones, navegación, transiciones y fallbacks accesibles; documentar y verificar ambos temas y estados.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: coherencia del material de navegación y creación contextual.

## Impact

CSS, paleta visual y documentación, con comprobaciones focales en Chromium, WebKit y ventana nativa. Sin cambios de datos, operaciones Rust, dependencias ni ventana opaca. El checkpoint `affca8a` y el acabado aceptado del sidebar se preservan; el pedido posterior del usuario autoriza incluir la serie en un commit local conjunto, sin publicación remota, sync ni archivo automático.
