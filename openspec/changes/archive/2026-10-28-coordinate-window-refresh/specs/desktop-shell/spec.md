# Spec Delta

## ADDED Requirements

### Requirement: Actualización de vistas coordinada

El escritorio SHALL actualizar las vistas visibles al recuperar foco, al activarlas o ante una solicitud manual. MUST agrupar eventos repetidos por consulta, limitar a dos las lecturas simultáneas de actualización por workspace y conservar el cupo hasta que una operación iniciada termine realmente. Los cambios confirmados por otro cliente SHALL reflejarse al siguiente refresco, sin polling continuo.

#### Scenario: Ráfaga durante una lectura
- **WHEN** llegan veinte eventos equivalentes mientras una consulta está pendiente o en curso
- **THEN** las solicitudes previas a su inicio se agrupan y como máximo queda una consulta posterior para cambios ocurridos durante la operación; no se exceden dos lecturas físicas simultáneas por workspace

#### Scenario: Navegación durante una lectura
- **WHEN** una consulta queda en espera o termina después de cambiar vista, fecha, entidad, workspace o desmontar el panel
- **THEN** la lectura en espera se descarta y la respuesta tardía no publica filas, estado ni error en la identidad nueva

#### Scenario: Cambio confirmado por otro cliente
- **WHEN** CLI o MCP confirma un cambio y luego se recupera el foco o se solicita actualizar
- **THEN** la consulta vuelve a leer los originales confirmados sin escribir ni alterar revisiones

### Requirement: Resultados anteriores identificados durante refrescos

Home y las listas compatibles SHALL conservar sus resultados mientras se actualiza la misma consulta. Si falla MUST indicar que son anteriores y ofrecer reintento; al cambiar la identidad MUST limpiar los resultados previos. Una actualización exitosa SHALL volver a la primera página y avisar si se habían cargado páginas adicionales. MUST deshabilitar el cursor anterior durante la actualización y mientras los resultados estén sin verificar.

#### Scenario: Actualización exitosa con páginas adicionales
- **WHEN** una lista con varias páginas cargadas se actualiza por foco o por solicitud manual
- **THEN** mantiene las filas visibles mientras consulta, publica la primera página nueva y avisa que se pueden cargar las demás nuevamente

#### Scenario: Actualización fallida
- **WHEN** falla la lectura de la misma consulta
- **THEN** mantiene las filas anteriores identificadas como no actualizadas, presenta el error y el reintento, y no permite continuar con el cursor anterior

#### Scenario: Consulta diferente
- **WHEN** se cambia vista o fecha seleccionada mientras hay resultados o una lectura pendiente
- **THEN** no se muestran los resultados de la identidad anterior como si correspondieran a la nueva

### Requirement: Edición y tablero protegidos durante actualizaciones

El escritorio MUST conservar borradores, selección e historial de edición mientras se actualiza. Si una revisión externa difiere de un borrador MUST conservar el buffer y mostrar conflicto. Projects Board MUST deshabilitar movimientos mientras sus datos estén actualizándose o no verificados, y permitirlos nuevamente sólo cuando el tablero vigente cumpla las condiciones de integridad y cobertura existentes.

#### Scenario: Conflicto de edición externo
- **WHEN** otro cliente modifica una entidad con cambios locales pendientes y la ventana recupera foco
- **THEN** el borrador permanece intacto y se informa el conflicto sin sobrescribir ninguna revisión

#### Scenario: Tablero pendiente o con error
- **WHEN** el tablero se actualiza o falla su lectura
- **THEN** las tarjetas anteriores pueden seguir visibles identificadas como anteriores, pero sus movimientos permanecen deshabilitados hasta obtener datos vigentes y completos
