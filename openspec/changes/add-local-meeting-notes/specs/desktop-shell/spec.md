# Spec Delta

## ADDED Requirements

### Requirement: Biblioteca y componente de reuniones

El escritorio SHALL integrar Meeting Notes debajo de People y como página accesible
desde Notes, con New Meeting Note y This Week/All/By People/Calendar. El componente
SHALL ofrecer Notas/Transcripción/Conclusiones/Participantes, controles de captura,
edición de actores y fuentes temporales, con breadcrumb, temas, guardias y ancho
estrecho compartidos. MUST distinguir datos reales, procesamiento y funciones no disponibles.

#### Scenario: Después de la reunión
- **WHEN** se abre una reunión finalizada desde People o Notes
- **THEN** se accede al mismo original, a sus tramos/actores corregibles y a
  conclusiones/acciones revisables, o a un estado explícito de generación pendiente/fallida

#### Scenario: Borradores y teclado
- **WHEN** se navega desde notas pendientes o se usa el componente en ancho estrecho
- **THEN** las guardias conservan el buffer, los controles tienen nombres accesibles
  y la transcripción paginada no provoca overflow global ni bloquea la edición manual

### Requirement: Popup de inicio y estado global visible

El escritorio SHALL proponer iniciar al detectar una posible reunión sin robar
el foco de escritura, deduplicar avisos y mostrar indicador/tiempo/fuentes durante
captura. MUST permitir descartar/silenciar la sugerencia, pausar/finalizar desde
el componente o control flotante y resolver captura activa antes de cerrar.
La detección SHALL configurarse y funcionar sólo mientras Noteka está ejecutándose.

#### Scenario: Posible llamada
- **WHEN** audio/ventana de una aplicación compatible indican posible llamada
- **THEN** aparece una sugerencia contextual que no afirma certeza indebida ni
  inicia grabación; abrir solamente Zoom o un navegador no demuestra una reunión

#### Scenario: Cierre y privacidad del estado
- **WHEN** se intenta cerrar con captura activa o se pausa la reunión
- **THEN** se puede detener y conservar o cancelar el cierre, y Pausada significa
  que no se siguen capturando micrófono/audio, no sólo que se oculta el texto
