# Verificación del vidrio neutro

29/10/2026. Se revisaron Home y la búsqueda global en la ventana nativa oscura.
Los fondos de contenido, sidebar, barra y paneles flotantes son neutros; el color
primario permanece en iconos, foco y acciones. La búsqueda es más compacta,
con borde claro, y su backdrop deja ver más contenido. La portada ASCII mantiene
continuidad con el fondo de Home.

La compilación pasó con cero diagnósticos Svelte y el aviso previo de tamaño de
chunk. En WebKit pasaron la prueba de tema y cambio de colores primarios, 106
comprobaciones de menús y búsqueda, y 23 de ASCII/side peek. También se comprobó
el fallback sólido de contraste aumentado y el desenfoque limitado al panel.
El tema claro se verificó en WebKit, sin inspección nativa. La preferencia física
de transparencia reducida no se probó en macOS.

El sidebar y la barra usan gradientes estáticos. Los menús conservan desenfoque
local de 10 px; no hay filtro de pantalla completa. La búsqueda se actualizó el
02/11/2026 a un vidrio líquido con `blur(22px) saturate(1.45)`; ver
`docs/verification-search-liquid-glass.md`. No se comparó CPU/GPU o energía. La
ventana sigue opaca y el WebView no reproduce la refracción nativa de Liquid
Glass.

Propagación del 29/10: el mismo material se aplicó al menú de vistas de Tasks,
tooltips visibles, diálogos pequeños y barra de formato abierta. Breadcrumb,
barra de edición, navegación de Settings y marco de side peek usan acabado
neutro estático; sus cuerpos permanecen sólidos. Se observaron en la ventana
nativa oscura Home, el menú de Tasks, Settings y el breadcrumb de Tasks. Build
sin diagnósticos Svelte; en WebKit pasaron tema/acento, 106 comprobaciones de
menús, 23 de ASCII/side peek y 18 de Projects Board. No se repitió una revisión
visual nativa del tema claro ni se midió CPU/GPU. `docs/design-system.md` registra
la regla y `AGENTS.md` exige su lectura para nuevas pantallas.
La barra de formato abierta se añadió después de esas pruebas; la compilación
final pasó y el recorrido del editor sumó 18 comprobaciones en WebKit.
