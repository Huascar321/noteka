## MODIFIED Requirements

### Requirement: Catálogo de iconos SVG local

La interfaz SHALL usar SVG locales seleccionados de Icona.works, Fill cuando exista
y bold para controles sin Fill, con equivalentes semánticos documentados. MUST
conservar tamaño, color, etiquetas y claves existentes, imágenes/logos, documentos
y payloads, y fallback seguro para nombres desconocidos. La generación MUST fijar
revisión y conservar procedencia/licencias, sin catálogo completo, red remota ni
código de animación para mostrar iconos estáticos.

#### Scenario: Inicio y uso offline
- **WHEN** se inicia o navega por la aplicación sin pulsar el menú lateral
- **THEN** los iconos se presentan estáticos desde recursos locales, con tamaño/color y sin descargar iconos ni activar animaciones

#### Scenario: Datos conservados
- **WHEN** se muestra un logo importado o un nombre no reconocido
- **THEN** se mantiene el logo local o se usa el fallback, sin modificar la entidad

#### Scenario: Peso y detalles en ambos temas
- **WHEN** se muestran controles y figuras en tema claro u oscuro
- **THEN** usan Fill o trazo bold de Icona y los detalles recortados dejan ver el fondo, sin perder legibilidad ni colisionar entre instancias

### Requirement: Animación explícita y breve del menú lateral

Únicamente pulsar un botón lateral de sección, Search o Collections SHALL activar
un ciclo breve CSS de Icona cargado a demanda, con retorno exacto al SVG estático.
MUST conservar navegación/guardias y no animar por hover, selección de otra
procedencia, render ni timers recurrentes. Clics repetidos reinician un único ciclo;
desmontaje, pestaña oculta o movimiento reducido cancelan y liberan recursos.

#### Scenario: Clic y reposo
- **WHEN** se pulsa con ratón o teclado un botón lateral habilitado
- **THEN** sólo su icono reproduce brevemente y vuelve a reposo, sin animar el contenido de la pantalla

#### Scenario: Reducción o cancelación
- **WHEN** hay preferencia de movimiento reducido, se oculta la ventana o desaparece el botón durante la carga/reproducción
- **THEN** el icono permanece o vuelve a reposo y no queda una reproducción diferida activa
