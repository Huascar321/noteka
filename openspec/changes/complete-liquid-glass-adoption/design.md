# Design

## Context

Ver proposal y delta desktop-shell. La guía y `src/style.css` definen materiales existentes; los controles usan Svelte y mantienen lógica local y guardias propias.

## Goals / Non-Goals

**Goals:** extender recetas sin alterar operaciones, anclaje o accesibilidad, y aportar evidencia sintética reproducible por motor.

**Non-Goals:** cambios en datos, persistencia, contratos, dependencias o composición nativa Tauri/macOS.

## Decisions

- Reutilizar clases opt-in para campos, acciones y grupos, en vez de aplicar estilos globales a elementos HTML. Mantener bordes y foco explícitos y fallbacks sólidos.
- Aplicar blur sólo al popup flotante; los grupos de cabecera usan material estático sin filtro para no alterar menús fixed. La vista integrada sigue sólida.
- Separar lista de relaciones y flecha para permitir truncado y envoltura sin forzar dos filas con un destino.
- Eliminar capturas redundantes People/Tags y reutilizar la reserva compartida, preservando creación contextual. Unificar el icono + de 16 px en las acciones New, conservando la presentación discreta de las filas de lista.
- Registrar la modalidad efímera de interacción con listeners desmontables: ocultar el anillo por puntero y mostrarlo al usar teclado; conservar marcas de selección y arrastre, sin persistencia.
- Extender el generador con corpus sintético y estados explícitos; registrar Chromium y WebKit por separado y medir contraste sobre composición real.

## Risks / Trade-offs

- [Anclaje fixed o clipping] → probar los tres peeks y scroll, con indicador fuera del área desplazable.
- [Bajo contraste y fallback anulado] → medir píxeles compuestos y estilos calculados en ambos temas y motores.
- [Capturas obsoletas o datos privados] → fijar estado de escena y usar corpus temporal sintético con hashes.
- [Diferencias de WebKit nativo] → declarar el límite sin extrapolar desde headless.

## Migration Plan

Sin migración. Los cambios de presentación se pueden revertir junto con la guía y el atlas. No sincronizar ni archivar el cambio automáticamente.
