# Spec Delta

## ADDED Requirements

### Requirement: Respaldo y recuperación de reuniones activas

El workspace SHALL conservar sesiones/transcripciones y audio original de reuniones
en backups canónicos; modelos/binarios regenerables permanecerán externos. MUST
rechazar un backup mientras exista captura/audio pendiente de publicar con un error
busy explícito, o ejecutarlo tras una barrera verificable de finalización, sin
pausar/borrar una grabación implícitamente. La restauración MUST NOT reactivar captura.

#### Scenario: Backup con reunión abierta
- **WHEN** se pide backup durante captura activa o publicación pendiente
- **THEN** se informa busy antes de generar un respaldo aparentemente completo;
  después de finalizar y publicar se incluyen audio, relaciones y revisiones acreditadas

#### Scenario: Restaurar o migrar
- **WHEN** se restaura un backup compatible o se migra desde la versión anterior
- **THEN** identidades, documentos, archivos y datos desconocidos se conservan,
  y cualquier sesión antes activa queda recuperable sin encender dispositivos

## MODIFIED Requirements

### Requirement: Backup consistente

El sistema SHALL generar un backup con todos los datos confirmados de un mismo estado lógico, originales administrados y un manifiesto de versión e integridad, incluso si otro cliente tiene abierto el workspace. Una captura de audio activa o audio original pendiente de publicar SHALL devolver busy hasta finalizar su barrera, sin detenerla implícitamente.

#### Scenario: Backup mientras el workspace está abierto
- **WHEN** se solicita backup con un cliente abierto y escrituras confirmadas, sin captura de audio activa ni originales pendientes de publicar
- **THEN** el backup pasa su validación y contiene un estado consistente que puede restaurarse
- **AND** no incluye una escritura parcial

#### Scenario: Originales de adjuntos
- **WHEN** el estado confirmado referencia adjuntos originales
- **THEN** el backup incluye esos originales con hashes y la restauración rechaza faltantes o alterados
