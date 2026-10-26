# Design

## Context

HomePanel ya dispone de refreshPanel que vuelve a consultar cada HomeReader, conservando su vista. WorkspacePane conoce si una pestaña está activa o bloqueada. Los encabezados de sección permiten navegar sin el desplegable de Home.

## Goals / Non-Goals

**Goals:** quitar los dos controles solicitados y refrescar las lecturas con teclado de forma segura.

**Non-Goals:** recargar el WebView, cambiar atajos de otras páginas, modificar datos o implementar Goals.

## Decisions

- HomePanel instala un listener de keydown en captura durante su montaje y lo retira al desmontar. Sólo el Home activo intercepta ⌘R/Ctrl+R sin Shift/Alt y F5 sin modificadores.
- Siempre se previene la recarga nativa para ese atajo en Home; no se consulta durante un bloqueo, diálogo abierto, composición IME o repetición de tecla. blocked se pasa explícitamente desde WorkspacePane.
- El atajo reutiliza refreshPanel; los lectores mantienen vistas y el control de respuestas tardías/cupo físico existente. No se toca persistencia, pestañas o buffers de otras páginas.
- Se retiran Navegación y Actualizar Home del markup. Se expone aria-keyshortcuts en el landmark Home, conservando los encabezados navegables.

## Risks / Trade-offs

- Atajo de recarga del WebView → preventDefault en captura evita que el evento llegue antes a manejadores de contenido.
- Home inactivo o desmontado → verificación de active y retirada simétrica del listener.
- No hay suites solicitadas → se compila y revisan rutas/listeners/guardias; no se atribuye aceptación nativa nueva.

## Migration Plan

Sin migración ni escrituras de datos. Baseline externo, edición acotada, compilación, revisión de código, sincronización del delta y archivo. Revertir sólo las líneas de este cambio conservando las ediciones anteriores.
