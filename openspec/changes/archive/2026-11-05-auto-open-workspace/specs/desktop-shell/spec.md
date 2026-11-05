# Spec Delta

## ADDED Requirements

### Requirement: Reapertura automática del workspace

La interfaz MUST abrir automáticamente al iniciar el último workspace usado y SHALL conservar su ruta como preferencia local. Si no hay preferencia o la ruta guardada ya no contiene un workspace, MUST intentar el directorio de datos predeterminado. MUST mostrar la selección explícita sólo cuando no exista ningún workspace, la apertura falle o el usuario pida Change workspace.

#### Scenario: Apertura automática

- **WHEN** se inicia la aplicación y existe un workspace guardado o el del directorio de datos predeterminado
- **THEN** la interfaz abre ese workspace sin pedir una ruta y muestra Home con su contenido

#### Scenario: Preferencia persistente

- **WHEN** se crea o abre un workspace con éxito
- **THEN** su ruta se guarda como preferencia local y el siguiente arranque la reutiliza

#### Scenario: Ruta guardada no disponible

- **WHEN** la preferencia apunta a una ruta que ya no contiene un workspace
- **THEN** se intenta el directorio de datos predeterminado, sin presentar datos de otro workspace como propios

#### Scenario: Primer uso

- **WHEN** se inicia la aplicación y no existe ningún workspace en la preferencia ni en el directorio de datos
- **THEN** ofrece crear o abrir uno y muestra su ruta después de seleccionarlo

#### Scenario: Cambio explícito de workspace

- **WHEN** el usuario elige Change workspace en Settings
- **THEN** vuelve a la pantalla de selección y su elección se guarda para el siguiente arranque
