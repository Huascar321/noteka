## MODIFIED Requirements

### Requirement: Home local con cobertura visible

El escritorio SHALL ofrecer Home de ancho amplio con portada local, icono de casa y secciones Tasks, Notes, Projects, Tags, Goals y People en ese orden. MUST presentar Goals desde sus originales y ofrecer clima optativo sin bloquear las secciones locales.

#### Scenario: Workspace vacío y estructura offline
- **WHEN** se abre Home en un workspace vacío sin red ni modelos
- **THEN** aparecen composición, secciones, estados vacíos y creación local, mientras Goals muestra sus grupos vacíos y clima ofrece activar ubicación sin datos simulados ni consultas automáticas

#### Scenario: Navegación compacta y funciones diferidas
- **WHEN** se usan los encabezados de las secciones de Home y se consulta la cobertura de las pestañas de referencia
- **THEN** Tasks/Notes/Projects/Tags/Goals/People llevan a secciones reales y Content y auxiliares pendientes se identifican sin controles que aparenten ejecutar esas funciones

## ADDED Requirements

### Requirement: Clima optativo en Home

Home SHALL permitir activar la ubicación actual explícitamente y mostrar condiciones actuales y pronóstico de hoy y siete días siguientes desde Open-Meteo. MUST indicar la transferencia de ubicación, atribuir la fuente y preservar el funcionamiento offline de Home.

#### Scenario: Activación y selección
- **WHEN** el usuario pulsa Set location y concede el permiso
- **THEN** Home obtiene su ubicación y consulta el clima de sus coordenadas sin selección manual

#### Scenario: Sin selección
- **WHEN** no hay ubicación actual activada
- **THEN** Home no realiza consultas de clima ni ubicación sin interacción explícita

#### Scenario: Datos no disponibles
- **WHEN** falla una actualización
- **THEN** Home muestra el último pronóstico guardado con su fecha o un error recuperable si no existe

#### Scenario: Ubicación actual
- **WHEN** el usuario activa ubicación actual y concede permiso del sistema
- **THEN** Home obtiene la ubicación en macOS y actualiza periódicamente el pronóstico para ella

#### Scenario: Preferencia anterior de ciudad fija
- **WHEN** existe una ciudad fija guardada de una versión anterior
- **THEN** Home no la utiliza ni obtiene ubicación hasta que el usuario pulse Set location

#### Scenario: Ubicación denegada o no disponible
- **WHEN** se rechaza el permiso o falla la ubicación
- **THEN** se informa el error, se conserva la última caché disponible y se ofrece reintento

#### Scenario: Animación de condiciones
- **WHEN** Home muestra condiciones actuales o pronóstico diario
- **THEN** usa iconos locales animados según el código WMO, indica día o noche cuando se conoce y conserva atribución visible

#### Scenario: Movimiento reducido y Home oculto
- **WHEN** el sistema solicita movimiento reducido o Home no está visible
- **THEN** las ilustraciones permanecen legibles sin animación activa

#### Scenario: Franja adaptable
- **WHEN** Home muestra el clima en una ventana ancha o estrecha
- **THEN** presenta ubicación y estado actual junto a los ocho días cuando hay espacio, conserva fechas accesibles en ancho estrecho y mantiene controles y atribución

#### Scenario: Controles del clima discretos
- **WHEN** el puntero pasa por el clima o el teclado enfoca sus controles
- **THEN** aparecen la fecha de actualización, la atribución y las acciones, con transición salvo movimiento reducido; en dispositivos sin hover permanecen visibles
