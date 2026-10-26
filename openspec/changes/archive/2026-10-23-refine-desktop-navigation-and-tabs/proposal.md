# Proposal

## Why

El lateral muestra diecisiete destinos con igual jerarquía y la búsqueda ocupa una pantalla completa. El usuario pidió conservar únicamente Home, Tasks, Notes, Projects, Tags y People como enlaces laterales, una búsqueda contextual como la de su Notion Desktop, un acceso de icono a Collections y pestañas que conserven el trabajo al alternar. También pidió usar su portada de Notion y mejorar la animación ASCII con los videos claro/oscuro entregados.

## What Changes

- Separar el catálogo de destinos de los seis enlaces visibles. Mantener acceso a las vistas secundarias mediante el buscador y a Collections mediante un botón de icono junto a la búsqueda.
- Incorporar un único buscador contextual con resultados reales, filtros compatibles, vista previa de solo lectura, teclado y acceso a la búsqueda avanzada existente. La barra lateral y el botón `+` de pestañas usan ese mismo diálogo.
- Crear pestañas de sesión para secciones y entidades: `+` permite elegir un destino y abrirlo en otra pestaña. Alternar conserva buffers, selección, scroll y editor; cerrar pestañas o el workspace protege cada edición pendiente.
- Reproducir espaciado, colores, bordes, jerarquía y portada observados en Notion usando recursos locales. La portada original se recuperó con el perfil autorizado, sin copiar notas ni credenciales. Un selector discreto alterna foto y ASCII.
- Reemplazar el desplazamiento del texto estático por tiburones ASCII con proyección espacial, giros, cola articulada y variantes claro/oscuro; pausar fuera de vista, en pestañas inactivas y con movimiento reducido.
- Añadir movimiento breve y coherente en apertura del buscador, alternancia de pestañas, paneles y estados interactivos, respetando movimiento reducido y sin introducir esperas en operaciones.

No se agregan búsqueda semántica a la UI, servicios remotos, migraciones, importación de páginas personales ni cambios de reglas Rust. Las pestañas son efímeras de la sesión; no se guardan borradores en localStorage.

## Capabilities

### New Capabilities

Ninguna; se amplían capabilities existentes.

### Modified Capabilities

- `desktop-shell`: navegación primaria reducida, pestañas independientes, cierre agregado y portada local con animación accesible.
- `local-text-search`: búsqueda contextual compartida con vista previa y apertura en pestañas, conservando la búsqueda avanzada y los contratos léxicos actuales.

## Impact

Frontend Svelte/CSS, componentes de navegación/búsqueda/portada y documentación. Se reutilizan las operaciones Rust existentes; SQLite, API y MCP no cambian. La implementación debe conservar los cambios locales previos del checkout, incluidos iconos y colores de entidades. La verificación usa workspaces sintéticos y navegador efímero para producto; el perfil persistente autorizado se usa solamente como referencia visual de Notion.
