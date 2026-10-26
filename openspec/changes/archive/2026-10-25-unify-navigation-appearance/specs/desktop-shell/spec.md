## ADDED Requirements

### Requirement: Iconografía primaria coherente

Home, Tasks, Notes, Projects, Tags, Goals y People SHALL compartir la forma del
icono de su breadcrumb en el lateral, pestañas de sección, encabezados de Home,
páginas y accesos de creación de sección. MUST compartir el acento vigente,
conservar trazos sin rellenos contradictorios y mantener la animación sólo al
clic lateral. Los iconos y colores personalizados de registros MUST conservarse.

#### Scenario: Siete destinos y sus contextos
- **WHEN** se recorren los siete destinos y los encabezados integrados de Home, Tasks y Notes
- **THEN** cada destino usa su misma forma y color canónicos en todos esos accesos, sin cambiar los iconos originales de los registros

#### Scenario: Estado animado y reposo
- **WHEN** se pulsa un destino lateral o se pide movimiento reducido
- **THEN** el ciclo lateral retorna al mismo icono estático del breadcrumb o permanece estático, sin animar encabezados ni datos

### Requirement: Color primario local configurable

Settings → Preferences SHALL ofrecer color primario azul inicial, sugerencias
simples y una última opción personalizada con rueda circular de tono, saturación,
luminosidad y valor hexadecimal. MUST aplicarlo inmediatamente a iconos primarios,
acciones, selección, enlaces y foco en ambos temas y todas las pestañas sin
alterar buffers o datos. MUST adaptar tonos para contraste legible, persistir
solamente valores hexadecimales válidos aparte del workspace y recuperar azul
ante un valor inválido o inaccesible; un fallo al guardar conserva el color
aplicado con un aviso recuperable en inglés.

#### Scenario: Sugerencia y selección precisa
- **WHEN** se elige un color sugerido o se ajusta la rueda por puntero/teclado o un hexadecimal válido
- **THEN** la interfaz usa el nuevo acento de inmediato y Settings muestra la selección, sin consultas externas ni guardado de entidades

#### Scenario: Persistencia y entrada inválida
- **WHEN** se reabre con un valor válido, inválido o almacenamiento inaccesible, o se ingresa un hexadecimal incompleto
- **THEN** se restaura el color válido o el azul inicial; una entrada incompleta no reemplaza la selección vigente y el fallo de guardado es recuperable

#### Scenario: Temas, foco y ancho estrecho
- **WHEN** se cambia tema con una elección muy clara/oscura o se opera el selector a 420 px
- **THEN** controles y enlaces mantienen contraste, el foco es visible, la rueda admite teclado y no aparece desbordamiento global

### Requirement: Textos de interfaz en inglés

El escritorio SHALL presentar sus etiquetas, botones, nombres accesibles,
estados vacíos, ayuda y mensajes propios en inglés. MUST preservar contenidos,
títulos, valores importados e identidades del workspace en su idioma original;
la traducción de presentación no convierte valores persistentes ni altera
contratos del núcleo. Los mensajes diagnósticos originales del núcleo se
conservan como detalle cuando no hay una traducción acreditada.

#### Scenario: Interfaz y contenido en distintos idiomas
- **WHEN** se abren Home, secciones, buscadores, Settings, editor y herramientas sobre datos con títulos en español
- **THEN** los controles propios se muestran en inglés y los títulos/contenidos conservan sus valores originales

#### Scenario: Guardias conservadas
- **WHEN** una navegación o cierre requiere resolver un borrador o falla una operación
- **THEN** las acciones Save and continue, Discard y Cancel mantienen las guardias y el buffer, con estado de interfaz en inglés y diagnóstico disponible
