# Design

## Context

Projects ya tiene cinco estados, orden opcional y guardias de apertura/captura.
El usuario eligió arrastre/reordenamiento en Board; no pidió un lienzo libre.
Las operaciones batch existentes validan y aplican EntityUpdate con CAS, recibo,
hash del estado base global, scope y efecto dentro de una transacción SQLite.

## Goals / Non-Goals

**Goals:** movimiento con ratón y teclado, persistencia del orden/estado y feedback
honesto ante conflictos, conservando documentos y metadata.

**Non-Goals:** canvas libre, enlaces visuales, cambios en tareas/relaciones,
ordenar proyectos ocultos o introducir API/migración nuevas.

## Decisions

1. HTML drag/drop con origen interno por instancia, indicadores antes de tarjeta
   y al final de columna. Botón Mover accesible abre diálogo nativo con estado y
   posición. Los botones y referencias existentes conservan su apertura.
2. Adaptador de presentación calcula la secuencia destino y órdenes enteros.
   Lee payloads completos y conserva campos. Los únicos campos cambiados son
   state y board_order; sólo se envían actualizaciones que cambian el payload.
   Rust conserva validación de dominio y transacción; no se modifica su API.
3. Preview del lote, contraste posterior de IDs/revisiones/estados/orden del Board
   completo y apply con receipt. El hash base rechaza cambios después del preview.
   No hay escritura antes de la acción ni reintento mutante automático. Recargar
   todas las secciones Home al terminar mantiene los resúmenes coherentes.
4. Board carga hasta cien proyectos en una página (límite Home/batch existente).
   Movimiento sólo en Board completo y sin metadata no disponible; una página
   parcial lo informa. Active/Planned conservan navegación y presentación.
5. El componente cancela antes del apply si deja de estar activo o se destruye;
   una operación ya enviada conserva la atomicidad del núcleo. Errores se muestran
   y nunca sustituyen confirmación de guardado. El movimiento respeta reducción
   de animaciones, además de la alternativa con teclado.

## Risks / Trade-offs

- [Cualquier escritura concurrente cambia el receipt global] → informar conflicto,
  actualizar la lectura y permitir otra acción explícita, sin aplicar a ciegas.
- [Vista con más de cien proyectos] → declarar límite; no reordenar páginas con
  registros ocultos. Una solución de escala distinta requiere otro cambio.
- [Transporte interrumpido tras apply] → resultado no confirmado, releer el Board
  y evitar reenvío automático; el núcleo sigue aplicando todo o nada.

## Migration Plan

No hay cambio de formato. Compilar y revisar interacción sobre snapshot separado,
sin guardar movimientos de prueba en el workspace personal. Conservar evidencia
de presentación y estado leído. No ejecutar suites no solicitadas.
