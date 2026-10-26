## MODIFIED Requirements

### Requirement: Home local con cobertura visible

El escritorio SHALL ofrecer Home de ancho amplio con portada local, icono de casa y secciones Tasks, Notes, Projects, Tags, Goals y People en ese orden. MUST conservar franjas de clima/Goals como pendientes explícitos sin cifras o acciones ficticias, recursos remotos ni bloqueo de las secciones implementadas.

#### Scenario: Workspace vacío y estructura offline
- **WHEN** se abre Home en un workspace vacío sin red ni modelos
- **THEN** aparecen composición, secciones, estados vacíos y creación local, mientras clima y Goals indican su falta de implementación y no presentan datos simulados

#### Scenario: Navegación compacta y funciones diferidas
- **WHEN** se usan los encabezados de las secciones de Home y se consulta la cobertura de las pestañas de referencia
- **THEN** Tasks/Notes/Projects/Tags/People llevan a secciones reales y Content, Goals y auxiliares pendientes se identifican sin controles que aparenten ejecutar esas funciones

## ADDED Requirements

### Requirement: Encabezado Home y actualización por teclado

Home SHALL omitir el desplegable Navegación y el botón Actualizar Home y ofrecer actualización con ⌘R en macOS, Ctrl+R en Windows/Linux o F5. MUST consultar sólo el Home activo mediante sus lectores, conservar vistas/pestañas/borradores y prevenir recarga nativa. Bloqueos, diálogos, composición y repetición no inician consultas; el listener se retira al desmontar.

#### Scenario: Actualización del Home activo
- **WHEN** se pulsa un atajo admitido con Home activo y disponible
- **THEN** sus secciones vuelven a leer los originales manteniendo sus vistas; no se recarga la aplicación ni se modifica un documento y los dos controles retirados no aparecen

#### Scenario: Diálogo o pestaña inactiva
- **WHEN** Home está inactivo o se pulsa el atajo con un diálogo/bloqueo, composición o repetición
- **THEN** no inicia una actualización; con Home activo se evita la recarga nativa y, tras desmontar, no quedan listeners de esa instancia
