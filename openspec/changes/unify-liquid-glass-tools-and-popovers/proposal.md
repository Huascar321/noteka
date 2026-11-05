# Proposal

## Why

El usuario aprobó las cuatro mejoras propuestas para continuar el sistema Liquid Glass: barras de herramientas agrupadas, campo del buscador, menús/selectores y selector ASCII/Photo. Actualmente comparten vidrio de distintas etapas y las acciones de sección quedan visualmente sueltas.

## What Changes

- Agrupar iconos de herramientas de las secciones y Milestones en una cápsula neutra; New conserva su superficie independiente.
- Reutilizar el material de campo del sidebar en el input del popup de búsqueda, con foco visible.
- Compartir borde direccional, reflejos y radios en menús de página/apertura/bloques, iconos, relaciones y paneles flotantes de herramientas.
- Convertir ASCII/Photo en una cápsula segmentada con indicador deslizante.
- Conservar operaciones, semántica, cierre, posicionamiento y alternativas accesibles; verificar ambos temas, acentos y ventana estrecha.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: agrupación y coherencia material de controles y superficies flotantes.

## Impact

Componentes Svelte, acción de indicador existente, CSS y documentación. Sin cambios Rust, formatos de datos, dependencias ni ventana nativa. El contenido y los controles previamente aceptados conservan su tratamiento; el pedido posterior del usuario autoriza el commit local conjunto de la serie, sin push, sync ni archivo automático.
