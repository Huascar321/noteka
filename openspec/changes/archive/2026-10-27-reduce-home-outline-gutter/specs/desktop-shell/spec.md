## ADDED Requirements

### Requirement: Margen compacto de contenido de Home

Home SHALL reservar un margen reducido junto a su indicador de secciones para aprovechar el ancho disponible. El indicador, scrollbar, contenido y borde ajustable del Side peek MUST permanecer accesibles y sin superponerse a controles, también en ventana estrecha y ambos temas.

#### Scenario: Home junto a un Side peek
- **WHEN** se muestra contenido de Home con un Side peek abierto o se modifica su ancho
- **THEN** el margen lateral es más estrecho, el contenido utiliza el espacio recuperado y las marcas, scrollbar y borde siguen operables

#### Scenario: Vista estrecha y temas
- **WHEN** se reduce el ancho de la ventana o se cambia entre temas claro y oscuro
- **THEN** el contenido y el indicador conservan separación, sin overflow global ni pérdida de acceso por teclado
