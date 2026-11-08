## ADDED Requirements

### Requirement: Continuidad líquida del calendario Due

El calendario Due de Task SHALL expandir su material desde la geometría del campo con asentamiento elástico y regresar al campo al cerrar. El contenido MUST aparecer legible sin la compresión geométrica del material. MUST conservar anclaje, selección, guardado explícito, teclado y foco, sin escrituras ni red por abrir o cerrar.

#### Scenario: Apertura y cierre contextuales
- **WHEN** se abre Due por puntero o teclado y se cierra por Escape, selección, campo o clic exterior
- **THEN** el material conecta campo y calendario; el cierre espera su transición y conserva las reglas de foco y edición existentes

#### Scenario: Interrupción y reposicionamiento
- **WHEN** se cierra durante la expansión o cambia el viewport, el scroll o la altura del mes
- **THEN** no salta a un panel completamente expandido antes de cerrar; el panel se mantiene anclado arriba o abajo y se ajusta al viewport cuando hay espacio suficiente para su contenido

### Requirement: Alternativas accesibles del movimiento Due

El calendario Due SHALL omitir movimiento decorativo y cerrar inmediatamente con movimiento reducido. MUST conservar superficies sólidas sin brillo ni blur al solicitar transparencia reducida o contraste aumentado. Al desmontarse MUST liberar animaciones y listeners.

#### Scenario: Preferencias de accesibilidad
- **WHEN** se abre el calendario en ambos temas con movimiento reducido, contraste aumentado o transparencia reducida
- **THEN** se aplica la alternativa correspondiente y se conservan controles, foco y selección
