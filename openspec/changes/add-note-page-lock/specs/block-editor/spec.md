# Spec Delta

## ADDED Requirements

### Requirement: Bloqueo de edición de Note con apertura temporal

La Note normal SHALL ofrecer `Lock page` con icono y switch en `…`. Sólo cuando
esa preferencia esté activa SHALL mostrar `Locked` con candado en la cabecera;
pulsarlo SHALL permitir edición temporal y mostrar `Re-lock`. Re-lock SHALL
restaurar el bloqueo y apagar Lock page SHALL quitarlo permanentemente y ocultar
el indicador. La apertura temporal MUST permanecer efímera por instancia de
página, conservarse al guardar/presentar la misma instancia y perderse al cerrarla
o navegar a otra entidad. Las páginas no bloqueadas MUST NOT mostrar el indicador.

#### Scenario: Bloquear y volver a abrir
- **WHEN** se activa Lock page sobre una Note guardada y se cierra/reabre
- **THEN** se confirma el estado persistido, se muestra Locked y no se puede editar título, cuerpo, icono, propiedades, relaciones, asociaciones externas ni archivar/restaurar; lectura, selección/copia, navegación y exportación siguen disponibles

#### Scenario: Edición temporal y re-bloqueo
- **WHEN** se pulsa Locked, se edita y guarda y se pulsa Re-lock
- **THEN** el cuerpo y controles quedan editables temporalmente sin apagar el switch persistido; al volver a bloquear se restauran sus restricciones y el documento guardado conserva IDs/contenido

#### Scenario: Desactivación permanente
- **WHEN** se apaga Lock page en el menú
- **THEN** sólo tras confirmar la escritura se quita el indicador y se permite editar también después de reabrir

#### Scenario: Buffer, error y conflicto
- **WHEN** hay cambios pendientes al bloquear/re-bloquear o falla la escritura de la preferencia por almacenamiento/revisión concurrente
- **THEN** se usan Guardar/Descartar/Cancelar sin perder el buffer; Cancel conserva el modo temporal y un fallo no afirma un nuevo estado persistido ni sobrescribe otra revisión

#### Scenario: Accesibilidad y contextos existentes
- **WHEN** se opera con teclado, temas claro/oscuro o ancho estrecho, o se abre una Note nueva, Task, Goal o Milestone
- **THEN** switch e indicador conservan nombres/estado/foco visibles y caben en el viewport; una Note nueva pide guardar antes de bloquear y los demás tipos conservan su comportamiento
