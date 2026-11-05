# Design

## Context

WorkspacePane ya tiene `unsaved-changes-dialog` y `unsaved-actions`. La acción modal conserva foco y la guardia resuelve Cancel/Discard/Save mediante funciones existentes. Save/Retry de cabecera usa un primario plano; New tiene tokens de vidrio teñido y el popup compartido está verificado. Ver `proposal.md` para la motivación.

## Goals / Non-Goals

**Goals:** aplicar el material aceptado y una jerarquía clara a las dos superficies mostradas.

**Non-Goals:** modificar guardias, validación, operaciones, conflictos, persistencia, contenido de textos o comportamiento de cierre.

## Decisions

- Reutilizar `.glass-popup` en la guardia, con el mismo blur local y marco, y un tinte de lectura algo más denso (69% claro/76% oscuro). Excluir los popups adoptados de la receta modal genérica para evitar que su mayor especificidad tape el material compartido. Conservar padding y límite de ancho; no añadir cierre al backdrop, porque la decisión pendiente requiere una acción explícita. Se descarta crear otro modal o reimplementar su foco.
- Usar una clase compartida de acción teñida para Save/Retry y Save and continue, con los tokens `--glass-action-*` de New, borde/reflejo y relieve al presionar. Mantener tamaño compacto de cabecera y botones de 36 px en la guardia. No se extiende a primarios fuera de este alcance.
- Cancel/Discard comparten cápsula de vidrio neutro de navegación, con iconos/etiquetas intactos. Mantener el orden y distinguir Save por el tinte primario; no introducir rojo decorativo que cambie su jerarquía. En ancho estrecho, secundarios comparten fila y confirmación ocupa la siguiente.
- Reutilizar las capas de borde sin eventos y definir fallbacks después de los materiales. Contraste del texto blanco se mide compuesto en reposo/hover, con más de un acento. Disabled y foco conservan sus estados existentes.
- Sólo clases/estilos; Rust y SQLite siguen siendo dueños de reglas/datos. Sin migración, dependencias ni nuevas escrituras.

## Risks / Trade-offs

- [Acción primaria sobre material claro] → usar tinte derivado legible de New y medir la composición, no sólo el color base.
- [Diálogo estrecho] → verificar filas, etiquetas y teclado a 420 px.
- [Guardado fallido o en curso] → probar recuperación y deshabilitados con el puente real antes de marcar completado.
- [WebKit compone distinto] → conservar fallbacks y registrar los límites de composición conocidos, sin afirmar equivalencia nativa.

## Migration Plan

Build/HMR local, con cambios previos preservados. Commit local conjunto autorizado posteriormente, sin push, sync ni archivo automático; sin cambio de formato o recuperación persistente.
