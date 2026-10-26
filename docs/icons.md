# Iconos de interfaz

Noteka usa **AnimateIcons** (`@animateicons/vanilla` **0.1.0**) con 81 SVG locales
seleccionados. La familia principal es Huge; las formas sin equivalente usan
Lucide del mismo paquete. El catálogo estático no incluye instrucciones de
animación ni importa el reproductor. No usa fuentes de iconos, CDN ni red remota.

Al 26/10/2026 se agregaron save/trash para el diálogo de borrador y
sidebar-right/app-window para los modos de peek. Se conservan las formas
anteriores y nueve definiciones laterales diferidas.

## Presentación y datos

`AppIcon` conserva `currentColor`, tamaño, grosor de línea predeterminado 1.8 y
fallback. Las claves lógicas previas siguen funcionando, por ejemplo `star`,
`file-text` y `project`. Los nombres desconocidos muestran el documento por defecto.
Los iconos decorativos llevan `aria-hidden`; los botones conservan sus nombres.

Los campos opcionales `icon`/`icon_color` y el selector existente siguen vigentes.
Rust valida `[a-z0-9-]`, hasta 40 bytes, y `#RRGGBB`. Una clave
`image-<UUID hexadecimal sin guiones>` conserva su adjunto local original con
[los límites del lector de imágenes](tasks-reference.md). El cambio de catálogo
no reescribe entidades, logos, relaciones ni documentos. No acepta SVG arbitrario
procedente de una entidad.

## Movimiento

Los iconos permanecen estáticos al renderizar, cambiar de pantalla o pasar el mouse.
Sólo un clic de ratón o teclado en Home, Tasks, Notes, Projects, Tags, Goals, People,
Search o Collections del lateral carga el reproductor diferido e inicia un ciclo.

El ciclo termina a los **800 ms** restaurando exactamente los atributos estáticos.
Otro clic reinicia un único ciclo. La preferencia de movimiento reducido, ocultar
la página o desmontar el botón cancela la reproducción. La navegación conserva
sus guardias; un fallo al cargar decoración no impide abrir una sección.

## Generación y licencias

- `scripts/icon-catalog.mjs` selecciona las formas y mantiene las claves.
- `npm run icons:generate` produce `icons.generated.ts` y las nueve definiciones
  laterales separadas usando `restingAttributes` oficial. Valida tags/atributos,
  referencias locales y namespace de IDs por instancia.
- `npm run icons:check`, incluido en build, contrasta los archivos generados y
  las licencias, copiadas también a `public/licenses/animateicons/` para distribuirlas
  dentro de los assets offline de la aplicación.
- `@lucide/svelte` dejó de ser dependencia; los SVG estáticos no necesitan React
  ni componentes de otra biblioteca.

[Documentación oficial vanilla](https://animateicons.in/icons/docs/vanilla),
[licencias y procedencia](../third-party/animateicons/README.md),
[ADR0041](decisions/0041-static-svg-and-explicit-icon-motion.md) y
[alcance de revisión](verification-static-animateicons.md).

## Desarrollo con Tauri

El paquete ESM de AnimateIcons se excluye del prebundler de Vite para evitar URLs
optimizadas obsoletas cuando se actualizan dependencias durante una sesión abierta.
Se conserva la carga al clic y el empaquetado diferido de producción. Un fallo
decorativo queda registrado en consola; no bloquea navegación ni cambia datos.
La [corrección y revisión geométrica](verification-icon-motion-dev-fix.md) cubre
el Vite activo en Chromium/WebKit, además de la revisión previa de producción.

People añadió seis claves estáticas el 24/10/2026: contact, circle-user, user-plus,
phone-call, arrow-up y arrow-down. Goals incorpora trophy ya existente como novena
definición lateral diferida; los 76 SVG estáticos anteriores permanecen idénticos.

## Consistencia de navegación

Las siete secciones usan una sola definición mediante `SectionIcon` y
`navigation.ts`, compartida por lateral, breadcrumb, pestañas de sección,
encabezados de Home/páginas y captura. Las formas canónicas son Home/home,
Tasks/task, Notes/file-text, Projects/project, Tags/tag, Goals/trophy y
People/person. Se retiraron circle-check/edit/rocket/circle-user de esos
encabezados, sus rellenos y flechas superpuestas; el catálogo y las imágenes
personales se conservan.

Color y trazo parten del breadcrumb: acento compartido, sin relleno, trazo 1,8.
El color primario se elige en [Settings → Preferences](settings.md); un ajuste
por tema mantiene contraste para selecciones extremas. El movimiento lateral
conserva su ciclo y reposo canónico; encabezados/breadcrumb siguen estáticos.

[Revisión de apariencia](verification-appearance.md).
