# Design

## Context

Los campos de Task y Project fijan padding lateral cero y las acciones de relaciones heredan un margen de una regla antigua. Las mismas filas se comparten entre tipos de entidad y pueden contener varios vínculos.

## Goals / Non-Goals

**Goals:** corregir causas compartidas y otros problemas observados sin alterar controles, gestos ni datos; revisar temas, anchos, contenido largo y accesibilidad.

**Non-Goals:** rediseño general, garantías para toda combinación posible, composición Tauri nativa o cambios de dominio.

## Decisions

- Usar padding lateral equilibrado en los campos compactos de fecha, preservando los controles nativos y el ancho disponible.
- Alinear la etiqueta con la primera línea de relaciones y eliminar el margen heredado de sus botones; mantener wrapping y targets.
- Revisar el atlas y componentes compartidos, complementar con recorridos sintéticos de geometría y registrar el alcance real. Usar una ruta ficticia visible para Settings durante capturas sin modificar archivos de imagen.

## Risks / Trade-offs

- [Elementos nativos y wrapping] → verificar ambos motores, fecha editable, filas vacías y pobladas y anchos estrechos.
- [Evidencia obsoleta] → regenerar atlas y mediciones afectadas con fuentes finales y comprobar hashes.
