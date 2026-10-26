# Proposal

## Why

Los siete destinos primarios usan distintas formas y colores entre lateral,
breadcrumb y encabezados. El azul está fijado en varios estilos y parte de la
interfaz sigue en español, lo que impide una apariencia y un idioma coherentes.

## What Changes

- Compartir los iconos canónicos del breadcrumb en Home, Tasks, Notes, Projects,
  Tags, Goals y People, incluidos encabezados, accesos de creación y pestañas.
- Unificar su color y el acento de controles/enlaces/selecciones con una
  preferencia local de color primario; azul inicial, sugerencias y selector
  circular de tono con controles de saturación/luminosidad y hexadecimal.
- Agregar ese ajuste en Settings → Preferences, con aplicación inmediata,
  persistencia validada y contraste en los dos temas.
- Traducir textos propios de la interfaz al inglés, incluidos nombres accesibles,
  acciones, estados y ayuda; conservar títulos/contenidos/propiedades importados.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: iconografía primaria consistente, acento configurable local
  y textos de interfaz en inglés sin transformar datos.

## Impact

Presentación Svelte/TypeScript/CSS y documentación. Sin migraciones, cambios de
contratos o reglas Rust, dependencias, modelos ni proveedores. Las preferencias
se guardan en localStorage aparte del workspace y respetan sus fallos recuperables.
La implementación está autorizada por la solicitud actual; Meeting Notes sigue
propuesto y fuera de este cambio.
