# Spec Delta

## ADDED Requirements

### Requirement: Página Tags con navegación y vistas locales

El escritorio SHALL presentar Tags lateral con icono sobre título, breadcrumb
Home → Tags, Nav a Tasks/Notes/Projects, captura New Tag y Recent inicial. SHALL
ofrecer Recent, Fav., A-Z, Types y Table con identidades originales, última nota
relacionada y clasificación disponibles. New page y Load more SHALL ocupar su
fila completa, acumular páginas y conservar las guardias de captura/navegación.

#### Scenario: Entrada a Tags y enlaces locales
- **WHEN** se abre Tags con datos locales disponibles
- **THEN** Recent muestra hasta diez filas iniciales, última nota y tipo, el tag
  y la última nota abren sus registros originales y Nav permite abrir las tres secciones

#### Scenario: Creación y continuación protegidas
- **WHEN** se solicita New Tag, New page o Load more
- **THEN** las dos primeras preparan una captura sin guardar automáticamente y
  la última acumula la página siguiente; navegar con cambios pendientes conserva
  Guardar/Descartar/Cancelar

### Requirement: Tabla y actualización de la página Tags

Tags SHALL ofrecer Table como tabla semántica con tag, última nota y tipo,
usando el orden A-Z y la paginación locales. SHALL reutilizar filtros y validación
Rust, las guardias de respuestas tardías y reintento, y actualización activa con
⌘R/Ctrl+R/F5. La selección Table SHALL conservarse al actualizar o continuar.
La página SHALL funcionar en ambos temas y ventana estrecha sin overflow global,
sin alterar entidades/documentos ni las variantes de Tags en Home y Notes.

#### Scenario: Tabla local acumulable
- **WHEN** se selecciona Table, se actualiza o se continúa la lectura
- **THEN** se muestran columnas accesibles con datos locales disponibles, se
  mantiene Table y se respeta el cursor y desempate por identidad del orden A-Z

#### Scenario: Lectura fallida o sustituida
- **WHEN** una lectura falla o llega después de cambiar de vista o cerrar la página
- **THEN** se conserva el reintento o se descarta la respuesta obsoleta con las
  guardias compartidas; ningún campo inválido ni ausencia de relación se inventa

#### Scenario: Temas y preservación
- **WHEN** se abre la página sin conexión o con ancho estrecho en claro u oscuro
- **THEN** los datos y originales siguen siendo locales, las pestañas y tabla
  pueden desplazarse internamente y el documento no adquiere overflow horizontal
