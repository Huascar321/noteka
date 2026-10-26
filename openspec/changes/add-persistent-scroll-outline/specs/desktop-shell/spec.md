# Spec Delta

## ADDED Requirements

### Requirement: Indicador persistente de secciones al desplazar

Home y Note normal SHALL mostrar marcas persistentes junto al borde derecho,
sin hover, resaltando la sección del scroll local. Home MUST representar Tasks,
Notes, Projects, Tags, Goals y People en su orden visual; Note, su título y
H1/H2/H3 visibles del documento. MUST excluir títulos de herramientas/vistas
incrustadas y mantener una marca de título en notas sin encabezados.

#### Scenario: Recorrer Home
- **WHEN** se desplaza Home desde la portada hasta el final
- **THEN** permanecen sus seis marcas; la marca activa sigue las secciones y People queda activa al alcanzar el final, sin requerir hover

#### Scenario: Recorrer y editar una Note
- **WHEN** una Note tiene encabezados repetidos o de distintos niveles y se agrega, renombra, mueve o elimina uno
- **THEN** el indicador mantiene orden/niveles y nombres actuales sin confundir títulos iguales; desplazar, editar o cambiar el tamaño actualiza el resaltado

#### Scenario: Nota sin encabezados o formato desconocido
- **WHEN** la Note no tiene encabezados reconocidos o sólo permite recuperación del documento
- **THEN** el título ofrece una marca local sin convertir el formato ni generar contenido, escrituras o estado pendiente

### Requirement: Navegación local y aislada del indicador

Cada marca SHALL permitir desplazarse a su título con nombre accesible, tooltip y
foco visible, en ambos temas y ancho estrecho. MUST respetar movimiento reducido,
conservar buffer/selección/documento, no activar operaciones de contenido y no
usar recursos externos. El indicador SHALL corresponder a su panel visible,
conservar el scroll de pestañas independientes y recalcularse ante cambios de
layout, paginación o tamaño, sin cubrir controles de contenido ni el scrollbar.

#### Scenario: Saltar dentro de un borrador
- **WHEN** se activa una marca por ratón o teclado con cambios pendientes
- **THEN** sólo cambia el scroll de esa página y el indicador, manteniendo contenido, revisión y guardias; la acción no guarda ni descarta

#### Scenario: Pestañas y contenido dinámico
- **WHEN** se alternan pestañas con diferentes posiciones o cambia la altura por cargar filas, plegar grupos o redimensionar
- **THEN** sólo se muestran indicadores de paneles visibles, se conserva el scroll previo y se actualizan posiciones sin referencias a otra pestaña; un Side peek y su origen tienen indicadores independientes

#### Scenario: Temas, movimiento y muchos títulos
- **WHEN** se usa tema claro/oscuro, 420px, movimiento reducido o una Note con muchos encabezados
- **THEN** las marcas y su foco siguen accesibles dentro del viewport, el indicador puede desplazarse internamente y el salto evita animación cuando corresponde
