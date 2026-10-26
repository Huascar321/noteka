# Verificación de alineación y transición del lateral

Fecha: 25/10/2026. Cambio `align-titlebar-and-animate-sidebar`.

## Implementación y compilación

- Barra de 44 px con pestañas de 32 px centradas, sin padding superior de 6 px;
  cierre centrado mediante grid y borde completo de pestaña.
- Dos columnas desktop estables, 250→0 px, y lateral de ancho fijo trasladado
  con la misma curva de 260 ms. Visibilidad diferida al cerrar; inert inmediato.
- Panel estrecho y fondo permanecen montados durante el cierre y transicionan
  también al abrir. La regla global de movimiento reducido anula las transiciones.
- Configuración Mac y=16→24, conservando x=14. Tao/Wry instalados calculan la
  posición del contenedor con y: el cambio lo desplaza 8 px hacia abajo.
- `npm run build` aprobado: cero errores/advertencias Svelte; aviso previo de
  Vite por bundle mayor a 500 kB conservado. Sin dependencias ni Rust modificados.
- Configuración Mac combinada con la base válida contra schema Tauri instalado.
  Ajv con unicodeRegExp=false por la expresión previa del schema; sin quitar
  propiedades. Ajv advierte formatos no registrados, como en la revisión anterior.

## Revisión headless

UI compilada en Chromium, presentación Mac simulada y workspace sintético creado
mediante puente CLI existente. Sin acceder al workspace personal. **56
comprobaciones actuales aprobadas** en temas claro/oscuro, anchos 1280/760/420 px:

- Los siete elementos medidos (control lateral, pestaña, icono, texto, cierre,
  SVG de cierre y nueva pestaña) tienen centro y=21,5 px en ambos temas.
- Colapso/expansión desktop con posiciones intermedias, ancho lateral constante
  y borde derecho alineado con el inicio del contenido durante el movimiento.
- Inversión antes de terminar sin perder estado final, panel original conservado
  y ninguna operación de entidad causada por los cambios de presentación.
- Apertura/cierre del panel estrecho interpolados, comienzo bajo la barra en
  y=44, fondo funcional y sin overflow global en 420 px. Breakpoint 760 correcto.
- Escape devuelve foco al control; navegación inerte inmediatamente al cerrar;
  fondo final oculto y sin interceptar el contenido.
- Movimiento reducido inmediato en desktop y estrecho, en ambos temas.
- Edición sintética sin guardar conserva el mismo input/panel/texto al plegar,
  expandir, pasar por otra pestaña y cancelar la guardia de cierre.
- Contexto web real separado: platform=web, sin reserva Mac ni controles falsos.

Tres intentos de controlador se conservan con sus resultados: clic en el centro
del fondo superpuesto caía dentro del lateral de 240 px; lectura inmediata tras
resize no esperaba el evento de matchMedia; selector exacto de pestaña omitía
el nombre accesible Unsaved changes. Se corrigieron el punto de clic, la espera
y el selector del controlador, sin cambios adicionales de producto. Los primeros
52 checks y los cuatro restantes aprobados se conservan juntos en final-report.json.
No se observaron errores JavaScript durante el recorrido instrumentado.

Evidencia temporal fuera de Git:
`/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/noteka-headless-RPdENx/`.
Capturas de revisión en `output/playwright/noteka-titlebar-{light,dark}.png` (ignoradas).

## Límites y cierre

La alineación HTML está medida; la posición final de los botones nativos Mac,
arrastre/fullscreen y fluidez física requieren aceptación en el host. No se usó
la pantalla del usuario ni se reinició su app. El ajuste de posición nativa se
aplica al próximo inicio normal; una recarga del frontend no lo aplica.
Windows real no ejercitado. No se añadieron suites permanentes ni se ejecutaron
las suites históricas. No hay cambios de datos, contratos, modelos o proveedores.

Dos requisitos desktop-shell modificados, manteniendo sus cinco escenarios
heredados y añadiendo dos sobre transición/inversión y movimiento reducido.
El número de requisitos permanece en 71. Meeting Notes queda propuesto 0/19.

Cierre: 3/3 tareas, dos deltas contrastados íntegramente con main y archivo
`2026-10-25-align-titlebar-and-animate-sidebar`. Los otros 69 requisitos, cinco
escenarios heredados, las otras specs y los archivos de Meeting Notes permanecen
idénticos al baseline previo a esta sincronización.
