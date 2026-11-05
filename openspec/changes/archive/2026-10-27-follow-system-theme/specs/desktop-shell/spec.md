## ADDED Requirements

### Requirement: Tema de apariencia del sistema

Settings SHALL ofrecer System como primera opción de Theme, junto a Dark y Light. Al seleccionar System, Noteka MUST aplicar la apariencia clara u oscura informada por el dispositivo y actualizarla mientras la aplicación esté abierta. Las opciones fijas MUST ignorar cambios del sistema. La elección SHALL persistir localmente sin cambiar datos o descartar buffers; ante fallos de almacenamiento se conserva la elección de la sesión y se informa el error. Valores anteriores de Light y Dark MUST conservarse; ante valor ausente, inválido o inaccesible se conserva el fallback oscuro.

#### Scenario: Sistema cambia en tiempo real
- **WHEN** está seleccionado System y el dispositivo cambia entre claro y oscuro
- **THEN** la interfaz y su paleta de acento cambian al tema correspondiente sin reiniciar ni alterar ediciones

#### Scenario: Tema fijo y reapertura
- **WHEN** se selecciona Light o Dark y cambia la apariencia del dispositivo o se reabre Noteka
- **THEN** se conserva el tema seleccionado independientemente del sistema

#### Scenario: Persistencia y fallo
- **WHEN** se selecciona System y se reabre la aplicación, o no se puede guardar la preferencia
- **THEN** se restaura System si se guardó; ante fallo se aplica durante la sesión y se informa el error recuperable
