# Spec Delta

## ADDED Requirements

### Requirement: Cabecera de Tag con propiedades y contador canónico

El detalle Tag SHALL presentar portada decorativa local, icono/título, Type/Favorite editables y Note Count desde el resumen Rust. MUST conservar las cuatro clasificaciones Area/Resource/Entity/Topic sin convertirlas, mostrar carga/error/ausencia sin inventar cero, y mantener procedencia/actividad de lectura en View details. El contador SHALL permitir llegar a Notes sin alterar el borrador. PageTools y sus condiciones/teclado MUST conservarse; sin cambios de datos al abrir.

#### Scenario: Datos locales y borrador
- **WHEN** se abre un Tag con resumen disponible y se modifica Type o Favorite
- **THEN** el contador refleja la consulta canónica y los controles preparan sólo el payload pendiente; navegar mantiene Guardar/Descartar/Cancelar y no altera documentos ni relaciones

#### Scenario: Resumen y tipos no reconocidos
- **WHEN** falla el resumen, falta el Tag en la página consultada o se encuentra una clasificación no reconocida
- **THEN** se ofrece reintento o ausencia explícita sin un contador ficticio y el valor desconocido se conserva hasta una selección explícita válida

### Requirement: Contenidos del Tag con contexto preservado

Tag SHALL presentar Nav a Notes/Tags y secciones Notes/Clips de workflow/context, distinguiendo Clips sólo por el tipo local web_clip y conservando títulos/identidades. MUST mantener tareas relacionadas y su captura, otras relaciones en cabecera y el editor completo de relaciones. Las listas SHALL acumular IDs únicos con su cursor común, distinguir cobertura parcial/carga/error y usar captura contextual/apertura protegidas. Temas, teclado y 420 px SHALL conservar accesibilidad sin overflow global ni controles de jerarquía/remotos inventados.

#### Scenario: Captura y navegación
- **WHEN** se abre un registro relacionado o se prepara Note/Clip/Task desde el Tag
- **THEN** se abre su original con guardias o se prepara la captura vinculada sin escribir; Clip sólo preselecciona el tipo existente, sin descargar ni extraer contenido

#### Scenario: Páginas y errores
- **WHEN** se continúa una página, falla la lectura o llega una respuesta tras salir
- **THEN** se acumulan originales únicos, se ofrece reintento sin afirmar ausencia total o se descarta la respuesta anterior; los paneles de herramientas conservan disponibilidad y teclado
