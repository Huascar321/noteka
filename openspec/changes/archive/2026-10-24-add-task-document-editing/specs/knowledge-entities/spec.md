# Spec Delta

## ADDED Requirements

### Requirement: Documento opcional de tarea con identidad estable

El núcleo SHALL admitir un documento enriquecido opcional en Task mediante create/update compartidos, con el mismo ID, revisión y relaciones. Una Task anterior sin documento SHALL seguir sin contenido persistido al abrirla o modificar sólo propiedades; document_editable SHALL ser true con cuerpo ausente y false ante un formato almacenado no editable. MUST reutilizar validación de versiones/bloques/referencias/límites, CAS, auditoría, índice textual, exportación y backup existentes. Las otras entidades conservan su contrato y la ausencia de documento en una actualización no elimina contenido.

#### Scenario: Primer contenido y reapertura
- **WHEN** se guarda el primer documento de una Task con revisión vigente
- **THEN** se confirma un único cambio atómico en esa Task con sus bloques/IDs originales, y las lecturas compartidas, búsqueda textual, exportación y backup acceden a ese cuerpo sin crear una Note asociada

#### Scenario: Task anterior y propiedades
- **WHEN** se abre una Task sin cuerpo o se actualiza su título/estado omitiendo documento
- **THEN** puede comenzar contenido pero sigue sin fila documental nueva hasta que el usuario guarda su cuerpo; no se cambian relaciones ni fechas omitidas

#### Scenario: Conflicto o formato desconocido
- **WHEN** se envía una revisión vencida, documento inválido o se intenta reemplazar un cuerpo almacenado con formato desconocido
- **THEN** se rechaza sin escritura parcial, el original permanece recuperable y no se normalizan ni descartan propiedades ajenas
