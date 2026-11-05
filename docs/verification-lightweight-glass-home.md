# Verificación del marco de Home inspirado en vidrio

29/10/2026. El primer ajuste aplica gradientes estáticos al sidebar y a la barra
de ventana, e indicadores sutiles al elemento activo, búsqueda y cabeceras Home.
La vista real oscura en macOS se inspeccionó visualmente sobre el workspace del
usuario sin modificar contenido. No se altera la geometría ni se introduce una
ventana transparente: el acabado aproxima la maqueta sin reflejar el escritorio.

Seguimiento del 29/10: el contenido oscuro usa una base azul grisácea y Home
un gradiente estático. La portada ASCII tiene fondo transparente y comparte el
fondo del contenido; la portada fotográfica mantiene su imagen. Se observó la
continuidad de Home y la búsqueda en la ventana nativa oscura. No se midió el
consumo de CPU o GPU, ni se revisó visualmente el tema claro en ventana nativa.
En WebKit, la prueba de portada pasó 23 comprobaciones, incluida la continuidad
de fondo en ambos temas y la pausa y reanudación de la animación.

Seguimiento posterior: el sidebar incorpora reflejos con gradientes estáticos y
los tintes nuevos del contenido, Home y sidebar se calculan desde el color
primario. Se revisó el sidebar oscuro en la ventana de Noteka. En WebKit se
comprobaron cambios de azul a violeta y verde desde Settings, temas claro y
oscuro, contraste del elemento activo y fallback sólido con contraste aumentado;
el recorrido de apertura y ajuste del sidebar pasó 12 comprobaciones y el de
ASCII/side peek pasó 23. No se midió CPU ni GPU; la transparencia es simulada y
no refleja el escritorio.

Build pasó con cero errores y advertencias Svelte y el aviso habitual de tamaño
de chunk. El recorrido de tema comprobó claro/oscuro, contraste mínimo 4,5:1 en
texto de navegación activo, ausencia de `backdrop-filter` en las dos superficies,
acento, búsqueda y borrador. El recorrido WebKit del sidebar pasó 12 checks con
movimiento reducido, resize, navegación y borrador. `prefers-reduced-transparency`
incluye fallback sólido cuando el WebView expone la preferencia; no se verificó
el ajuste físico del sistema ni la apariencia nativa clara.

No se añadieron filtros de desenfoque, capas animadas, temporizadores ni consultas.
Es una elección orientada a reducir trabajo gráfico; no se midió CPU, GPU ni
consumo energético antes/después, por lo que no se cuantifica el impacto real.
La búsqueda global y los menús flotantes no se rediseñaron en esta etapa.
