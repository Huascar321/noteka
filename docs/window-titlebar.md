# Barra superior y lateral plegable

Actualizado el 25/10/2026. La barra de Noteka reúne el control del lateral, las
pestañas abiertas y `+` en una fila de 44 px, por encima de navegación y contenido.
El lateral puede plegarse también en escritorio; la pestaña conserva su panel,
borrador, revisión, historial y scroll. El estado del lateral es de sesión.

El control lateral, la pestaña, sus iconos/texto/cierre y `+` comparten centro
vertical. La tira usa pestañas de 32 px centradas en la barra; los controles Mac
tienen posición configurada `(14, 24)`, aplicada al próximo inicio del host.

Expandir y contraer usa una transición de 260 ms con desaceleración, reversible
desde su posición actual. En escritorio el contenido acompaña el desplazamiento
del lateral de 250 px; en ancho estrecho se desliza el panel de 240 px y se
desvanece el fondo. Las etiquetas mantienen su ancho. La navegación que se cierra
queda `inert` inmediatamente; con movimiento reducido el cambio es inmediato.
Ver [verificación del ajuste](verification-titlebar-alignment.md).

## Plataformas

| Entorno | Composición |
|---|---|
| macOS | Controles rojo/amarillo/verde del sistema, título nativo oculto y contenido superpuesto; espacio reservado a la izquierda del control del lateral. |
| Windows | Lateral y pestañas a la izquierda; minimizar, maximizar/restaurar y cerrar a la derecha, con ventana sin decoración duplicada. |
| Web | Barra interna sin controles de ventana ni acciones nativas. Esto no añade almacenamiento ni backend web. |
| Otro host de escritorio | Barra interna conservando su decoración del sistema. |

Los espacios libres marcados permiten arrastrar la ventana en Mac y Windows.
Los botones de pestaña, cierre, `+` y lateral no son zonas de arrastre.
Los controles Windows usan APIs de la ventana principal y permisos individuales;
Cerrar llama a la autoridad compartida que recorre todas las pestañas.
Cancelar o fallar un guardado conserva el workspace y sus borradores.
No se promete Snap Layout al pasar el puntero sobre un botón HTML de maximizar.

## Ancho estrecho y teclado

Hasta 760 px, el lateral es un panel superpuesto que se abre desde el mismo
control. Escape cierra ese panel y devuelve el foco al botón; también se puede
cerrar con el botón o el fondo. La navegación oculta usa `inert` y `aria-hidden`.
Cambiar entre anchos cierra el panel superpuesto y conserva la selección de
plegado de escritorio. Las pestañas se desplazan dentro de su tira y mantienen
los atajos de [navegación](desktop-navigation.md).

## Aplicar la decoración

La configuración nativa se lee al crear la ventana. Una recarga del frontend
por sí sola no aplica `titleBarStyle` ni `decorations`: hace falta un nuevo
inicio del host. Este cambio no cierra ni reinicia manualmente la sesión nativa
del usuario. Antes de reiniciarla, resolver sus borradores con las guardias habituales.

## Límites y evidencia

Ver [revisión](verification-window-titlebar.md) y
[ADR0047](decisions/0047-integrated-window-titlebar.md).
La compilación del host Mac no acredita el funcionamiento de arrastre ni
los controles del sistema. Las vistas por plataforma en Chromium son
previsualizaciones de composición; Windows real sigue pendiente, como pidió
el usuario. No se capturó la pantalla nativa.

## Referencias

- [Personalización de ventanas Tauri](https://v2.tauri.app/learn/window-customization/).
- [Configuración por plataforma](https://v2.tauri.app/develop/configuration-files/#platform-specific-configuration).
- [WindowConfig](https://v2.tauri.app/reference/config/#windowconfig).
