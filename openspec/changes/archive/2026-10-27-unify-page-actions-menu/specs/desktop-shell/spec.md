## ADDED Requirements

### Requirement: Menú de acciones compartido en detalles

Los detalles y borradores de Note, Task, Project, Tag, Person, Goal y Milestone SHALL presentar sus herramientas y acciones disponibles en un único menú `…` de cabecera. MUST conservar las capacidades por tipo, disponibilidad y guardias de borrador, sin duplicar el menú o ejecutar operaciones al abrirlo. Lock page SHALL limitarse a Note normal hasta que exista soporte en otros tipos.

#### Scenario: Páginas y herramientas
- **WHEN** se abre cualquier detalle o borrador y se despliega el menú
- **THEN** aparecen las herramientas aplicables con las mismas condiciones de disponibilidad, y Archive o Restore para entidades guardadas; no se repiten las herramientas al pie

#### Scenario: Teclado, tamaño y modos de detalle
- **WHEN** se usa el menú en un panel estrecho, Side, Center o Full y se navega con teclado
- **THEN** sus opciones y paneles se mantienen dentro del viewport y permiten apertura, navegación y cierre con retorno de foco

#### Scenario: Acciones con borrador pendiente
- **WHEN** se elige Archive o Restore con cambios pendientes
- **THEN** se conserva la guardia para guardar, descartar o cancelar y la revisión esperada de la entidad; abrir o cerrar el menú no modifica datos
