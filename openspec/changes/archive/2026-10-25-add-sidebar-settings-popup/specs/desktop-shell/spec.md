# Spec Delta

## ADDED Requirements

### Requirement: Popup Settings desde el lateral

El lateral SHALL sustituir el selector de tema, ruta y acción de cambio de
workspace por un botón Settings con icono de engranaje a la izquierda. MUST
retirar «En tu equipo» y el punto azul junto a la marca. Settings SHALL abrir un
popup modal con esos tres controles, conservando el tema local, sus errores
recuperables y la ruta original como lectura. Abrir/cerrar o cambiar tema MUST
conservar entidades, buffers, pestañas y posiciones sin escrituras de dominio.

Settings SHALL organizarse en navegación y contenido por secciones: Preferencias
para apariencia y General del workspace para ruta/cambio. La búsqueda local MUST
filtrar los ajustes disponibles, llevar a su sección, mostrar un vacío explícito
y permitir limpiar la búsqueda. La composición SHALL adaptarse al ancho estrecho
y permitir añadir categorías sin presentar controles futuros inexistentes.

El popup SHALL ofrecer cierre accesible por botón y Escape, contener el foco y
devolverlo al acceso anterior al cerrarse. MUST funcionar offline en ambos temas
y ancho estrecho, impedir atajos de navegación detrás del popup y cerrar antes
de iniciar las guardias existentes de cambio de workspace para todas las pestañas.

#### Scenario: Acceso y contenido agrupados
- **WHEN** se abre un workspace y se pulsa Settings en el pie del lateral
- **THEN** aparece un popup con Tema, ruta y Cambiar workspace; el pie contiene sólo Settings con su engranaje y la marca no tiene punto azul ni el texto retirado

#### Scenario: Tema local y recuperación de preferencia
- **WHEN** se cambia tema desde Settings, incluso con edición pendiente o almacenamiento de preferencias inaccesible
- **THEN** el tema se aplica sin escribir ni descartar la edición; se conserva su persistencia local y se muestra el error recuperable si no pudo guardarse

#### Scenario: Secciones y búsqueda local de ajustes
- **WHEN** se navega entre Preferencias y General o se busca un ajuste existente o desconocido
- **THEN** se muestran únicamente la sección y controles que coinciden, se puede limpiar un resultado vacío y no se consultan proveedores ni se modifican datos o buffers

#### Scenario: Cambio de workspace protegido
- **WHEN** se pulsa Cambiar workspace con cambios en una pestaña activa o inactiva
- **THEN** Settings se cierra y el flujo existente considera todas las pestañas; cancelar o fallar un guardado conserva workspace y contenido pendiente

#### Scenario: Teclado y ancho estrecho
- **WHEN** se usa Settings en cualquier tema o ancho estrecho y se cierra por botón o Escape
- **THEN** el foco permanece dentro mientras está abierto, los atajos no actúan sobre pestañas detrás del modal, el contenido no desborda y el foco retorna al acceso anterior disponible
