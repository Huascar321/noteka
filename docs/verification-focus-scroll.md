# Verificación del scroll durante el refresco

29/10/2026. En Chromium headless con UI compilada, CLI real y workspace sintético,
el caso de foco con una lectura retenida ajustó `scrollTop` de 1014 a 1234 px
antes de la corrección mientras Projects permanecía aproximadamente en el mismo
sitio por anclaje del navegador. Los mensajes transitorios agregaban altura en
las secciones. Tras mover el aviso a la barra y restaurar foco sin scroll, el caso
verifica posición y scroll sin cambios mayores a un píxel, además del foco de un
control fuera de pantalla.

Build aprobado con cero diagnósticos Svelte y el aviso habitual de tamaño de
chunk. El recorrido focal pasó 21 comprobaciones tanto en Chromium como WebKit,
incluyendo datos anteriores, errores, paginación, guardado y conflicto con
borrador. Una ejecución intermedia falló por timeout al intentar editar la nota;
una repetición posterior y la ejecución final pasaron. El foco de macOS físico
no se reprodujo nuevamente. Cambios reales de contenido o el reinicio de páginas
pueden modificar la altura y no se fuerza la posición contra el scroll del usuario.
