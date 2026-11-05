# Verificación: alineación visual

La revisión posterior a Liquid Glass examinó las 71 escenas sintéticas por motor
(claro/oscuro, búsquedas, herramientas, navegación, popups y detalles) y las
reglas compartidas de campos, relaciones y acciones. Se identificaron padding
lateral nulo en fechas de Task/Project, márgenes heredados en acciones y botones
de relaciones, y botones de preparación de adjuntos centrados respecto de una
etiqueta con campo en vez del selector. WebKit además mostraba los selectores
compactos con 21 px pese al mínimo CSS. Se corrigieron los estilos compartidos,
con altura explícita de 32 px para selectores compactos, manteniendo sus controles
nativos, nombres y operaciones.

- Build: aprobado, sin errores ni advertencias Svelte; Vite mantiene la
  advertencia por tamaño del chunk principal.
- Geometría: 92 checks por motor en 1440, 900 y 420 px y ambos temas, sobre
  Task, Project, Note, Tag y Person; incluye relaciones vacías y pobladas,
  nombres largos, alineación de etiquetas y botones de eliminación, padding y
  valor de fechas, overflow y la alineación de las acciones de adjuntos.
- Recorridos Liquid Glass: 569 checks Chromium y 568 WebKit; foco 21 y
  herramientas 13 por motor, aprobados con el código actualizado.
- Una primera prueba WebKit detectó que el selector nativo medía 21 px; se
  corrigió su altura explícita y se repitió la geometría completa en ambos
  motores. El primer intento de la prueba no se descarta como éxito.

- Las cuatro matrices Blue/Violet de contraste compuesto aprobaron 2.164
  muestras en total, mínimo 4,503:1; se comprobaron además los fallbacks de las
  escenas instrumentadas. El detalle y las omisiones están en la
  [verificación Liquid Glass](verification-complete-liquid-glass-adoption.md).
- El atlas final se regeneró con 71 escenas por motor y los manifiestos y
  hashes de capturas, fuentes y build se contrastaron con el checkout final.

El atlas es evidencia de escenas concretas, no prueba de todas las vistas,
estados, longitudes de contenido o configuraciones de usuario posibles. La ruta
temporal que aparecía en Settings fue reemplazada antes de la captura por una
ruta ficticia, sin editar las imágenes. Las pruebas usan CLI/SQLite sintéticos y
motores headless, sin IPC ni composición nativa de Tauri/macOS.
