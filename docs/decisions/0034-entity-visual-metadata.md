# ADR 0034: Apariencia de entidad en el payload

- Fecha: 23 de octubre de 2026
- Estado: adoptado para metadata; catálogo sustituido por [ADR0041](0041-static-svg-and-explicit-icon-motion.md)
- Ámbito: `note`, `task`, `project`, `tag` y `person`

## Contexto

La interfaz necesitaba iconos SVG consistentes y la posibilidad de elegir un
icono y un color por página, en lugar de usar emojis. Guardar esa elección sólo
en el navegador la perdería al cambiar de equipo o restaurar un backup, y no
permitiría que CLI/MCP vieran la misma apariencia. El usuario pidió además un
catálogo open source y que el color funcione con el tema claro/oscuro.

## Decisión

1. Se adopta Lucide (`@lucide/svelte`, ISC) como catálogo de iconos de interfaz.
   Los componentes heredan `currentColor` y no descargan recursos.
2. Los payloads de las cinco entidades built-in aceptan dos campos opcionales:
   `icon` (clave Lucide, `[a-z0-9-]`, máximo 40 bytes) e `icon_color`
   (`#RRGGBB`). Se serializan sólo si están presentes; los payloads anteriores
   siguen siendo válidos sin migración.
3. El núcleo valida ambos campos en `productivity::payload`. Las claves y colores
   arbitrarios se rechazan; el SVG nunca se persiste.
4. La UI muestra un selector de 24 iconos y nueve colores en la página. La
   elección forma parte del payload de la entidad y se guarda con la revisión
   normal. `HomeItem` recibe `icon`/`icon_color` en `EntityRef`.
5. Los iconos personalizados del export de Notion no están disponibles en el
   Markdown; las páginas importadas usan el icono por defecto de su tipo hasta que
   el usuario elija uno.

## Alternativas

- Guardar la elección en `localStorage`: no es fuente de verdad, no viaja en el
  backup y no es visible para CLI/MCP. Descartado.
- Crear una tabla y operaciones nuevas de apariencia: aislaría el payload, pero
  añade migración, operaciones y joins para un dato pequeño. Se prefiere el campo
  opcional dentro del payload existente.
- Guardar el SVG o CSS: amplía la superficie de ejecución y no hace falta para
  iconos de línea. Descartado.

## Consecuencias

- Cambio aditivo de contrato: las huellas de esquemas previos se recalcularon
  porque `NoteFields`, `TaskFields`, `ProjectFields`, `TagFields` y
  `PersonFields` incorporan campos opcionales. No se eliminó ni reinterpretó
  ningún campo existente y no hubo migración SQLite.
- El color explícito no sustituye el contraste del tema; si se omite, el icono
  usa `currentColor`.
- El catálogo se limita a las claves incluidas en `src/lib/icon-set.ts`. Cambiar
  el catálogo exige actualizar la validación de iconos si se agregan claves fuera
  del patrón permitido.

## Evidencia

`docs/icons.md`, pruebas Rust del núcleo, `npm run check`, build y recorridos
headless/nativos con capturas.
