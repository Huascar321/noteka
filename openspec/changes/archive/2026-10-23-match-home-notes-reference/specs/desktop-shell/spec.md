# Spec Delta

## ADDED Requirements

### Requirement: Notes con composición de la referencia

Notes SHALL presentar encabezado, New Note, pestañas con iconos y filas compactas con arte, relaciones, tipo y antigüedad disponibles. MUST conservar apertura y creación protegidas, accesibilidad por teclado, temas, errores y cobertura de página sin cargar imágenes remotas.

#### Scenario: Notas y referencias reales
- **WHEN** se abre Notes en claro u oscuro
- **THEN** las filas muestran originales y vínculos locales, tipos como chips y días compactos con procedencia accesible; una imagen ausente usa fallback

#### Scenario: Lista estrecha o lectura fallida
- **WHEN** se reduce el viewport o falla una consulta
- **THEN** título y controles siguen accesibles sin overflow global, el error ofrece reintento y no se muestra como lista vacía

### Requirement: Vistas diferenciadas de Notes

Rust SHALL calcular Notes, Fav., Clips, Voice, Journal y Meetings además de Inbox y All. MUST filtrar antes de paginar, separar la biblioteca de otras páginas importadas mediante procedencia explícita y conservar caps, cursores y descarte tardío. All mantiene acceso a todas las notas locales activas.

#### Scenario: Biblioteca y categorías
- **WHEN** hay notas locales y páginas importadas de varias colecciones
- **THEN** Notes incluye notas locales y notas generales de la colección Notes no archivadas en origen; las categorías usan tipo/favorito explícitos y All conserva todas las páginas activas

#### Scenario: Paginación y metadata inválida
- **WHEN** se pagina o hay metadata no interpretable
- **THEN** cada vista conserva orden/desempate estable y cobertura no disponible; un cursor de otra vista se rechaza y las respuestas anteriores no reemplazan la vista actual

### Requirement: Recuperación explícita de metadata de notas

La recuperación de metadata y arte SHALL usar el export/manifiesto o referencias explícitas autorizadas, con backup y revisión esperada. MUST omitir coincidencias ambiguas, conservar documentos, relaciones existentes y contenido sensible fuera de Git, sin abrir cuerpos de notas ni inventar iconos por título.

#### Scenario: Fuente e identidad inequívocas
- **WHEN** una nota coincide con el manifiesto original y las propiedades exportadas
- **THEN** se recuperan sólo los campos acreditados y vínculos visibles inequívocos, guardando informe externo y los documentos originales intactos
