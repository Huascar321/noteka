# Spec Delta

## MODIFIED Requirements

### Requirement: Backup consistente

El sistema SHALL generar un backup con todos los datos confirmados de un mismo estado lógico, originales administrados y un manifiesto de versión e integridad, incluso si otro cliente tiene abierto el workspace.

#### Scenario: Backup mientras el workspace está abierto
- **WHEN** se solicita backup con un cliente abierto y escrituras confirmadas
- **THEN** el backup pasa su validación y contiene un estado consistente que puede restaurarse
- **AND** no incluye una escritura parcial

#### Scenario: Originales de adjuntos
- **WHEN** el estado confirmado referencia adjuntos originales
- **THEN** el backup incluye esos originales con hashes y la restauración rechaza faltantes o alterados
