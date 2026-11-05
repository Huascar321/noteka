# Spec Delta

## ADDED Requirements

### Requirement: Ayuda inferior de atajos retirada

El editor compartido MUST NOT mostrar el texto «/ for blocks · ⌘Z to undo» en ninguna de sus variantes. SHALL conservar los comandos de bloques y el historial existentes.

#### Scenario: Abrir un editor
- **WHEN** se abre cualquier variante del editor compartido
- **THEN** la ayuda inferior indicada no aparece y el comportamiento de edición permanece disponible
