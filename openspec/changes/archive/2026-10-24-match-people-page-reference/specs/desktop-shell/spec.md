# Spec Delta

## ADDED Requirements

### Requirement: Página lateral People de referencia

El escritorio SHALL abrir People lateral con icono sobre título, Nav a Contacts,
Pipeline y calendario local, New Person y las siete vistas existentes; Contacts
SHALL ser inicial y usar galería con fotos originales, relaciones y empresa.
MUST reutilizar consultas, guardias, breadcrumb canónico, temas, reintento,
actualización activa y paginación acumulativa sin modificar datos al navegar.

#### Scenario: Contactos y navegación local
- **WHEN** se abre People y se activan Contacts, Pipeline, Cal o las otras vistas
- **THEN** se muestran sus datos locales y vacíos reales con las reglas Rust
  vigentes, y Cal presenta el calendario de cumpleaños sin depender de red o modelos

#### Scenario: Captura y ancho estrecho
- **WHEN** se crea o abre una persona, se carga otra página o se reduce el ancho
- **THEN** las identidades y fotos siguen siendo originales, la creación conserva
  Guardar/Descartar/Cancelar y los controles/títulos son accesibles en ambos temas
  sin overflow global ni reemplazo por lecturas obsoletas
