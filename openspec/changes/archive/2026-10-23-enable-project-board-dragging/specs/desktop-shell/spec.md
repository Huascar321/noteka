# Spec Delta

## ADDED Requirements

### Requirement: Movimiento de tarjetas de Projects Board

Projects Board SHALL permitir arrastrar una tarjeta antes de otra o al final de una columna, tanto dentro de su estado como entre estados. MUST ofrecer alternativa de teclado con elección de estado/posición, indicadores y estado de guardado/error, sin iniciar un movimiento desde datos de arrastre externos ni impedir abrir el proyecto o sus referencias.

#### Scenario: Orden y estado
- **WHEN** se suelta un proyecto en una posición válida de Board o se confirma Mover con teclado
- **THEN** se conserva ese orden tras recargar y cambiar de columna cambia el estado; soltarlo en su misma posición no escribe datos

#### Scenario: Cancelación o página incompleta
- **WHEN** se cancela el arrastre, se cierra el diálogo o el tablero es una página parcial/no interpretable
- **THEN** no se inicia una escritura y la limitación se informa; el resto de navegación conserva sus guardias

### Requirement: Persistencia atómica del orden de proyectos

El movimiento SHALL usar las operaciones de lote existentes, con payloads completos conservados, revisiones esperadas y receipt del preview. MUST contrastar el snapshot completo del tablero antes del apply, rechazar conflictos sin reintento mutante automático y preservar títulos, documentos, iconos, procedencia y relaciones. Los estados y órdenes se validan en Rust; el orden de presentación no excede los límites actuales.

#### Scenario: Cambio concurrente o error
- **WHEN** el tablero cambia durante el movimiento o falla la operación
- **THEN** el apply se rechaza o revierte de forma atómica, se informa el error y se actualiza la lectura; no se presentan cambios parciales como guardados

#### Scenario: Ámbito completo
- **WHEN** Board contiene hasta cien proyectos interpretables y está cargado completo
- **THEN** las tarjetas se pueden mover usando sólo sus revisiones/IDs reales, sin actualizar documentos ni modificar registros fuera del orden afectado
