# Settings

El pie del lateral contiene el botón **Settings**, con engranaje estático a la
izquierda. Se retiran el punto azul junto a Noteka y el texto «En tu equipo».

El popup adopta la composición de la referencia aportada: navegación lateral,
búsqueda y contenido por secciones con encabezados, grupos y filas. Hoy ofrece:

- **Personal → Preferences → Appearance:** Theme ofrece System, Dark y Light.
  System sigue la apariencia del dispositivo en tiempo real. La preferencia
  local `noteka.theme.v1` conserva las elecciones anteriores; un valor ausente,
  inválido o inaccesible usa Dark. Si no puede guardarse, se mantiene durante
  la sesión y se muestra un error recuperable.
- **Personal → Preferences → Interaction:** Smooth mouse scrolling suaviza
  pasos de rueda reconocibles; está activado por defecto. Desmarcalo para volver
  al comportamiento nativo. La detección es conservadora, no identifica todos
  los dispositivos, y movimiento reducido mantiene el desplazamiento nativo.
- **Workspace → General:** ruta del workspace como lectura y Change workspace.
  La acción **Change workspace** cierra Settings y reutiliza la revisión de todas las pestañas, incluso
  inactivas, con Save and continue/Discard/Cancel y conservación ante fallos.

Buscar ajustes filtra esos controles y el color primario, sin distinguir mayúsculas/acentos,
y activa la sección correspondiente. Un resultado vacío ofrece limpiar la
búsqueda. No consulta al núcleo ni a proveedores y no cambia datos.

## Color primario

**Preferences → Appearance → Primary color** ofrece Blue (inicial), Violet,
Pink, Orange, Green y Teal; **Custom** es la última opción. Abre una rueda
circular de tono, sliders Saturation/Lightness y Hex color con Apply o Enter.
La rueda acepta arrastre, flechas, Shift y Home/End; todos los controles tienen
nombre y foco accesibles. No abre servicios ni un selector externo.

La clave local `noteka.primary-color.v1` admite solamente `#RRGGBB`, normalizado
a minúsculas. Ausencia, valor inválido o lectura inaccesible usa el azul
`#3183a6` del breadcrumb. Un error al guardar conserva el color aplicado y
muestra un aviso recuperable. No cambia el formato del workspace.

La selección actualiza iconos primarios, acciones, selección, franja de Home,
enlaces y foco en todas las pestañas. Los tonos se adaptan por tema: iconos
con contraste mínimo 3:1 sobre superficie/selección y enlaces, 4,5:1.
Por elección explícita del usuario, los botones primarios muestran texto y atajos
blancos sobre su fondo de acento actual, en ambos temas, en vez de la tinta
automática negra/blanca de la paleta. Los iconos y colores personales y los estados semánticos se
conservan. Tema y color se aplican al elegir, sin guardar ni descartar buffers.

## Interacción

El dialog modal contiene el foco con Tab/Shift+Tab, se cierra con el botón o
Escape y devuelve el foco al acceso anterior. Los atajos de pestañas/búsqueda
no actúan detrás del popup. Abrirlo no desmonta pestañas ni escribe sus buffers.
El tema se aplica inmediatamente; no hay un botón Guardar para preferencias.

En escritorio hay dos columnas con contenido desplazable. En ancho estrecho,
la navegación pasa arriba, la ruta larga se envuelve y el contenido conserva
scroll. Ambos temas y movimiento reducido usan los estilos compartidos.

## Ampliaciones

`SettingsDialog.svelte` separa metadatos locales de secciones y ajustes del estado
del tema/workspace que posee App. Una nueva categoría se incorpora a la navegación
y filtro junto con su panel real; no se muestran opciones futuras sin implementar.
Nuevas preferencias deben definir primero autoridad, persistencia y validación:
    este contenedor no añade un formato genérico a SQLite ni duplica reglas Rust.

El icono Settings viene del catálogo estático local existente, mediante
`scripts/icon-catalog.mjs` y su generador. No se añade una dependencia ni se
reproduce una animación al abrir el popup.

[Verificación anterior](verification-settings.md) y
[revisión de consistencia/color/inglés](verification-appearance.md).
