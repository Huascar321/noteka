# Design

## Context

Search es un botón flex de 32 px, con etiqueta, icono, atajo y apertura del diálogo existente. El sidebar tiene un fondo neutro estático. La referencia pide una cápsula más plana que la selección lateral; ver `proposal.md`.

## Goals / Non-Goals

**Goals:** replicar forma, gris tenue y borde iluminado interior sin alterar navegación ni densidad.

**Non-Goals:** convertir el botón en un campo editable, cambiar Collections o implementar las recomendaciones adicionales de vidrio.

## Decisions

- Mantener el marcado y la altura; radio completo y padding ajustado dentro de la misma caja. Conservar truncado y ocultación del atajo en sidebar estrecho.
- Definir tokens de campo `--glass-field-*` para tinte, borde, sheen, sombras y estados. Usar gradientes estáticos y borde sólido fino, sin blur sobre el sidebar ni el relieve fuerte de una selección activa.
- Conservar la tinta legible y el foco de la app. Hover/pressed modifican discretamente el material; los fallbacks van después de los estilos base y retiran gradientes y sombras.
- Sin operaciones nuevas, datos o migración. Rust y SQLite mantienen propiedad del contenido; sólo cambia CSS.

## Risks / Trade-offs

- [Contraste de la referencia] → conservar la tinta del tema y medir fondo compuesto, no copiar el gris del texto literalmente.
- [Recorte en ancho mínimo] → probar sidebar estrecho y contención de etiqueta/atajo.
- [Ventana opaca] → material aproximado sin refracción del escritorio; no afirmar rendimiento de GPU sin medirlo.
