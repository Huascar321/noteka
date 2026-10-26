# Spec Delta

## ADDED Requirements

### Requirement: Inicio visible del cuerpo de una Note

La Note normal SHALL ofrecer escritura directa, placeholder, Enter desde el título y estructuras Página vacía/Checklist/Nota con secciones sólo sobre un cuerpo vacío. MUST conservar IDs y prefijo importado al editar, usar el editor y guardado explícito compartidos y no sustituir cuerpos con encabezados, listas u otro contenido existente. Abrir o enfocar MUST mantener el estado guardado; las estructuras sólo preparan el borrador y no ejecutan operaciones de entidades.

#### Scenario: Inicio vacío
- **WHEN** se abre una Note sin bloques visibles o con un solo párrafo vacío y se presiona Enter desde el título
- **THEN** el editor recibe foco sin escritura persistente ni estado pendiente por el foco; las estructuras dejan de ofrecer reemplazo después de empezar

#### Scenario: Estructura y prefijo conservados
- **WHEN** se elige una estructura en una Note vacía importada o se edita una Note con código existente
- **THEN** se prepara sólo el cuerpo del borrador con IDs propios o se conserva el código editado, manteniendo todos los bloques del prefijo original intactos y las guardias/undo/redo existentes
