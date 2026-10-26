# Revisión de consistencia y preferencias

Fecha: 25/10/2026, America/La_Paz. Cambio `unify-navigation-appearance`.

## Resultado

Las siete secciones usan el icono del breadcrumb desde una sola definición:
Home/home, Tasks/task, Notes/file-text, Projects/project, Tags/tag,
Goals/trophy y People/person. Lateral, breadcrumb, pestaña de sección,
encabezados Home/página y captura comparten acento, trazo 1,8 y ausencia de
relleno. Se retiran las variantes y flechas superpuestas de los encabezados;
se conservan los iconos/colores personales y la animación al clic lateral.

Settings → Preferences → Appearance ofrece seis sugerencias y Custom al final,
con rueda circular, Saturation/Lightness y hexadecimal. El color se aplica a
iconos, acciones, selección, enlaces y foco en todas las pestañas. La preferencia
local nueva sólo admite #RRGGBB; azul #3183a6 inicial y ante ausencia/corrupción.
Los tonos se adaptan por tema con contraste mínimo 3:1 para iconos sobre
superficie/selección y 4,5:1 para enlaces y texto de botones, incluido hover.

Los textos propios de Svelte/TS se presentan en inglés: navegación, Settings,
editor, colecciones, herramientas, lectores, búsqueda, estados y ayudas.
Los vacíos de grupos Home se traducen por identidad del grupo, sin traducir
títulos. Se conservan contenido, propiedades, claves/marcadores importados,
alias de búsqueda en español y diagnósticos originales del núcleo como detalle.

## Evidencia de compilación y código

`npm run build` aprobado: 77 iconos/nueve definiciones laterales y contratos
vigentes; Svelte sin errores ni advertencias. Bundle final `index-BS7tPjLx.js` y
CSS `index-CtdicsY2.css`. Sigue el aviso previo de chunk mayor a 500 kB.

La comparación con la copia anterior confirma que 51 módulos sólo cambian
literales/textos de presentación manteniendo estructura AST. Los cambios
estructurales adicionales corresponden al componente compartido, preferencia,
Settings, metadatos de pestaña, etiquetas Home y locale explícito de fechas.
AppIcon, contratos/catálogos generados, lector de propiedades importadas,
escena/reloj/render ASCII y reproductor lateral se conservan exactamente.

Doce scripts UI existentes ajustan selectores/expectativas de etiquetas al
inglés y pasan `node --check`. No se agregan suites ni se ejecutan suites
históricas; no se atribuye nueva cobertura a esas adaptaciones de selectores.

## Revisión integrada

App compilado real servido por el puente de revisión existente, CLI Rust y
SQLite sintéticos. IPC/eventos Tauri simulados; no se usa pantalla ni workspace
personal. **56 comprobaciones actuales aprobadas**, en dos recorridos:

- **38:** forma/color/trazo de los siete destinos, breadcrumb, sugerencias,
  aplicación global, rueda por teclado/Shift/puntero, hexadecimal normalizado,
  error de entrada, fallo de persistencia y recuperación, inglés, temas/420 px,
  foco contenido/restaurado y originales/icono personal conservados.
- **18:** restauración al recargar, búsqueda en inglés y vacío de Settings,
  extremos blanco/negro/gris, borrador en pestaña inactiva conservado al cambiar
  color/tema, cambio de workspace protegido con acciones en inglés, cancelación
  sin escrituras, fallback por preferencia inválida y animación sólo lateral con
  restauración exacta del SVG canónico.

Las seis entidades y documentos sintéticos se comparan íntegros antes/después;
los cambios de apariencia y guardias canceladas no producen escrituras de
entidades. El recorrido principal registra cero errores de navegador y cero
solicitudes externas. Capturas de Settings oscuro/claro y 420 px, Home y Tasks
se revisan visualmente.

La revisión directa de **18 paletas** cubre nueve colores, incluidos negro,
blanco y amarillo, en ambos temas: contraste de icono/superficie/selección,
enlaces, texto/hover de botones y roundtrip hexadecimal/HSL. También verifica
preferencia ausente, normalización, lectura válida, corrupción y storage bloqueado.

## Correcciones intermedias y límites

Una revisión intermedia detectó contraste insuficiente en algunos iconos sobre
selección; se añadió ajuste acotado del tono y se repitió la revisión de las 18
paletas y el recorrido principal sobre el build final. Otra revisión buscaba el
nombre exacto del tab sin su indicador accesible Unsaved changes; se corrigió
sólo el selector y el recorrido de 18 comprobaciones pasó completo.
Los logs intermedios permanecen etiquetados como tales.

Evidencia y capturas privadas/ignoradas en `output/playwright/appearance-20261025/`;
copia de fuentes anterior y catálogo de traducción fuera del repositorio.
La compilación/revisión headless no acredita transporte nativo, pantalla física
o controles del sistema. No se modifica Rust, SQLite, contratos, modelos,
proveedores ni datos personales. Meeting Notes sigue propuesto 0/19.

## Cierre

Cuatro tareas completas; tres requisitos añadidos y 7 escenarios, con los
68 requisitos anteriores conservados íntegros (71 en desktop-shell). Delta
contrastado con main y cambio archivado como
`2026-10-25-unify-navigation-appearance`. La validación documental no sustituye
la compilación ni la revisión de producto descritas arriba.

Validación estricta final: 18 elementos aprobados, cero fallos.
