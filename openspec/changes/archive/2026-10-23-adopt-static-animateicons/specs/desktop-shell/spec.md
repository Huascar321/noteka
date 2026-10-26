# Spec Delta

## ADDED Requirements

### Requirement: Catálogo de iconos SVG local

La interfaz SHALL usar un catálogo seleccionado de AnimateIcons en SVG estáticos
locales para sus iconos de sistema, conservando tamaño, color, etiquetas accesibles
y claves lógicas existentes. MUST preservar imágenes/logos importados, documentos
y payloads; nombres desconocidos conservan un fallback seguro. La generación
reproducible MUST fijar versión y conservar licencias, sin cargar catálogos completos,
red remota ni código de animación para mostrar iconos estáticos.

#### Scenario: Inicio y uso offline
- **WHEN** se inicia o navega por la aplicación sin pulsar el menú lateral
- **THEN** los iconos se presentan estáticos desde recursos locales, con tamaño/color y sin descargar iconos ni activar animaciones

#### Scenario: Datos conservados
- **WHEN** se muestra un logo importado o un nombre no reconocido
- **THEN** se mantiene el logo local o se usa el fallback, sin modificar la entidad

### Requirement: Animación explícita y breve del menú lateral

Únicamente pulsar un botón lateral de sección, Search o Collections SHALL activar
un ciclo breve del icono pulsado, con reproductor cargado a demanda y retorno exacto
al SVG estático. MUST conservar la navegación/guardias y no animar por hover,
selección de otra procedencia, render ni temporizadores recurrentes. Clics repetidos
reinician un único ciclo; desmontaje, pestaña oculta o movimiento reducido cancelan
la reproducción y liberan listeners/temporizadores.

#### Scenario: Clic y reposo
- **WHEN** se pulsa con ratón o teclado un botón lateral habilitado
- **THEN** sólo su icono reproduce brevemente y vuelve a reposo, sin animar el contenido de la pantalla

#### Scenario: Reducción o cancelación
- **WHEN** hay preferencia de movimiento reducido, se oculta la ventana o desaparece el botón durante la carga/reproducción
- **THEN** el icono permanece o vuelve a reposo y no queda una reproducción diferida activa
