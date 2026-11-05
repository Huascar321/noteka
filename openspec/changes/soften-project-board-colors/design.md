# Design

## Context

El tablero combina acentos saturados con bordes y acciones teñidos en cada columna.

## Goals / Non-Goals

Suavizar la apariencia en claro y oscuro, manteniendo nombres y colores de estados reconocibles, y destino de arrastre visible. No cambiar la estructura, funcionalidad ni otras vistas.

## Decisions

Usar superficies y bordes neutros, color tenue en etiquetas, contadores moderados y acción New page neutral. Acotar estilos a `.project-board` para no modificar listas y conservar el foco visible.

Seguimiento: el usuario pide algo más de color. Las columnas mezclan un ocho por
ciento del acento de estado con la superficie; los badges recuperan los mismos
fondos y tintas por estado usados en la interfaz, manteniendo neutrales las
tarjetas y acciones.

## Risks / Trade-offs

Menos saturación no debe eliminar el contraste del texto ni del destino de arrastre. Verificar ambos temas y los indicadores existentes.
