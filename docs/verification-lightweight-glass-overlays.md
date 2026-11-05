# Verificación de búsqueda y menús de vidrio

29/10/2026. La búsqueda global y los menús de acciones, peek, editor, relaciones,
iconos y herramientas comparten una superficie translúcida con desenfoque de
6 px acotado al panel abierto, cuando el motor lo admite. El backdrop completo
de búsqueda ya no aplica el desenfoque de 2 px anterior. Transparencia reducida
o contraste aumentado dan fondo sólido sin filtro cuando el motor expone la
preferencia. No se usa una ventana transparente ni Liquid Glass nativo.

La app nativa en oscuro mostró búsqueda y menú de acciones con contenido real,
sin modificarlo. Build pasó con cero diagnósticos Svelte y el aviso habitual de
chunk. En Chromium y WebKit la prueba de portada y side peek pasó 21 checks por
motor: los cuadros se detienen con búsqueda abierta, reanudan al cerrar y siguen
activos junto al side peek. La prueba de menús en WebKit pasó 105 checks con
teclado, acciones, guardias, pantalla estrecha y fallback de contraste aumentado;
la de tema pasó en Chromium. La preferencia física de transparencia reducida y
la apariencia nativa clara no se verificaron.

La pausa elimina cuadros de portada durante el bloqueo y el filtro deja de cubrir
toda la pantalla; el nuevo filtro local aún puede tener costo gráfico. No se
compararon CPU, GPU, energía ni memoria antes/después en macOS, por lo que el
impacto total no está cuantificado. No se instaló ninguna dependencia de vidrio.

Seguimiento del 29/10: se ajustó la transparencia y el tinte de búsqueda y
menús y el desenfoque acotado pasó a 10 px. Se observó la búsqueda en la ventana
nativa oscura. En WebKit se verificó el contraste del texto secundario de la
búsqueda sobre extremos claros y oscuros, ambos temas, el fallback de contraste
aumentado y las interacciones; pasaron 106 comprobaciones de menús. El costo
de CPU/GPU no se comparó, y la apariencia nativa clara permanece sin revisión.
