# Spec Delta

## ADDED Requirements

### Requirement: Vidrio transparente propio de Search

Search SHALL mostrar el contenido real de la aplicación detrás mediante vidrio neutro, desenfoque corto y borde luminoso, sin velos blancos continuos en resultados y preview. MUST conservar dimensiones, búsqueda local, filtros, navegación, foco y transición existentes sin alterar Settings ni datos.

#### Scenario: Contenido visible en ambos temas
- **WHEN** se abre Search sobre Home en tema claro u oscuro y cambia el acento
- **THEN** las formas del fondo se perciben a través del panel con desenfoque, el material permanece neutro y los textos mantienen contraste de al menos 4,5:1 sobre el fondo compuesto

#### Scenario: Búsqueda y alternativas accesibles
- **WHEN** se buscan páginas, se abren filtros, se usa teclado o se reduce el ancho
- **THEN** resultados, preview y cierre siguen operativos sin overflow global; transparencia reducida o contraste aumentado restituyen fondos sólidos sin filtros ni reflejos, y movimiento reducido conserva la entrada inmediata
