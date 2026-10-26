# Spec Delta

## ADDED Requirements

### Requirement: Barra integrada de ventana por plataforma

Noteka SHALL reunir el control del lateral, las pestañas de sesión y nueva pestaña en una sola barra superior. En macOS SHALL conservar los tres controles del sistema con espacio propio y sin un título nativo duplicado; en Windows SHALL ofrecer minimizar, maximizar/restaurar y cerrar en el extremo derecho. MUST permitir arrastrar la ventana desde espacios libres, mantener interactivos los botones y conservar las guardias de cierre de todas las pestañas. En web SHALL omitir controles y acciones de ventana nativa. MUST distinguir configuración implementada de aceptación nativa pendiente, sin atribuir a una previsualización comportamiento del sistema operativo.

#### Scenario: Barra de Mac y espacios libres
- **WHEN** se abre Noteka en macOS y se usan sus pestañas o un espacio libre de la barra
- **THEN** el control del lateral y las pestañas comparten altura con los botones del sistema sin superponerse, los botones siguen siendo clickeables y el espacio libre permite mover la ventana

#### Scenario: Controles de Windows y edición pendiente
- **WHEN** se minimiza, maximiza/restaura o solicita cerrar desde la barra de Windows
- **THEN** se aplica la acción de ventana correspondiente, el estado maximizado tiene nombre accesible y cerrar resuelve los cambios de todas las pestañas mediante Guardar/Descartar/Cancelar antes de terminar el host

#### Scenario: Navegador y ventana estrecha
- **WHEN** se usa la variante web o un ancho estrecho en cualquiera de los temas
- **THEN** no aparecen controles nativos falsos en web, las pestañas se desplazan internamente y lateral, nueva pestaña y cierre de pestañas conservan nombres accesibles sin overflow horizontal global

### Requirement: Lateral plegable sin reconstruir pestañas

El control superior SHALL expandir o contraer el lateral en escritorio y abrirlo/cerrarlo como panel superpuesto en ancho estrecho. MUST reflejar su estado accesible, retirar del foco la navegación oculta y conservar las instancias, buffers, selección y scroll de todas las pestañas. Plegar el lateral o cambiar el ancho SHALL ser una operación de presentación sin lecturas ni escrituras de entidades ni reinicio de la aplicación.

#### Scenario: Plegar con una edición pendiente
- **WHEN** se alterna el lateral con una página o borrador abierto
- **THEN** el área principal aprovecha el ancho liberado y el mismo panel conserva su contenido, revisión y estado sin guardar, descartar ni recrear la pestaña

#### Scenario: Panel estrecho y teclado
- **WHEN** se abre la navegación en ancho estrecho y se cierra con su control o Escape
- **THEN** la navegación puede usarse por teclado, el foco vuelve al control al cerrar con Escape y los controles ocultos dejan de participar del orden de tabulación
