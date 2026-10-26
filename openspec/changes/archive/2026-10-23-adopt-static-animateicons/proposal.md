# Proposal

## Why

El usuario pidió reemplazar los iconos de interfaz por AnimateIcons, en SVG local
optimizado, estáticos salvo una animación breve al pulsar el menú lateral.

## What Changes

- Catálogo seleccionado de AnimateIcons con SVG estáticos generados y claves
  lógicas compatibles; iconos de acciones, vistas y navegación coherentes.
- Reproductor cargado a demanda únicamente al pulsar Search, Collections o una
  sección lateral; un ciclo breve, con reposo exacto y movimiento reducido.
- Generación reproducible, versión fijada y avisos de licencia conservados.
- Se conservan logos/imágenes importados y documentos. Sin animaciones hover,
  red remota de iconos, nuevas operaciones del núcleo ni migraciones.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: catálogo local estático y animación explícita del menú lateral.

## Impact

AppIcon, catálogo/generador, acciones laterales y pictogramas de controles.
Dependencia fijada `@animateicons/vanilla` 0.1.0; sin React ni cambios de datos.
La selección usa Huge y los equivalentes Lucide del mismo proveedor cuando una
forma no existe en Huge. La reducción de carga se medirá en el build local.
