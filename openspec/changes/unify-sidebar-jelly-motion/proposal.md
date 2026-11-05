# Proposal

## Why

El usuario prefiere el rebote Jelly actual de Tasks y pide aplicarlo por igual
a los nueve iconos del menú lateral.
Después pide un ritmo más lento y pausado: el ciclo pasa de700 a1000ms.

## What Changes

- Reutilizar la exportación CSS Jelly de Tasks con las nueve figuras existentes.
- Conservar keyframes e intensidad, con1000ms por ciclo y limpieza a1050ms, además de
  carga diferida, clic explícito, cancelaciones y reduced motion actuales.
- Conservar SVG estáticos, colores, etiquetas y datos. No agregar preferencias
  ni modificar la navegación o el reproductor.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: Jelly uniforme en los nueve accesos laterales.

## Impact

Catálogo/importador/generador local de Icona, recursos generados y documentación.
La receta oficial checklist/Jelly se exporta una vez y se aplica a un grupo que
contiene cada SVG estático; compartir CSS evita duplicar keyframes en el bundle.
Sin Rust, cambios de formato persistente, operaciones, dependencias o requests
remotos. Se verifican preservación de Tasks/catálogo estático, generación
reproducible, build y movimiento real en Chromium/WebKit headless.
