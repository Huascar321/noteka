# Spec Delta

## ADDED Requirements

### Requirement: Herramientas contextuales de edición en Notes

La Note normal SHALL ofrecer formato al seleccionar texto y un menú de bloques
`/` junto al cursor, filtrable por nombre con flechas, Enter y Escape. SHALL
ofrecer Text, Heading 1/2/3, listas, checklist, cita, código, divisor, adjunto local
y referencia a vista existente. MUST usar sólo tipos/marcas admitidos, conservar
selección al operar y permitir escribir barras literales sin convertirlas en bloques.

#### Scenario: Selección y formato
- **WHEN** se selecciona texto de una Note y se activa negrita, cursiva, tachado, código o enlace
- **THEN** se aplica la marca admitida a la selección y permite undo/redo; una URL inválida conserva el texto y ofrece un error

#### Scenario: Comandos con teclado
- **WHEN** se escribe `/head`, se navega con flechas y se elige un encabezado con Enter
- **THEN** el menú muestra coincidencias junto al cursor, reemplaza sólo el comando y conserva el resto del documento; Escape deja el texto escrito

#### Scenario: Contexto y lectura
- **WHEN** se escribe una URL, una barra dentro de código o se abre/enfoca una Note
- **THEN** no se convierte el texto ni se prepara una escritura persistente; adjuntos y vistas requieren selección explícita

### Requirement: Acciones sobre bloques completos de Notes

La Note normal SHALL mostrar controles laterales sólo por hover, foco explícito
de teclado o arrastre, para insertar debajo, convertir, duplicar, mover y quitar.
El hover SHALL abarcar la altura real del bloque y ambos márgenes horizontales
visibles de su propio panel de Note, sin extenderse a huecos verticales,
superficies superpuestas u otros paneles.
SHALL ofrecer arrastre interno y botones accesibles: mover conserva IDs y
duplicar los renueva también en hijos. MUST preservar atributos, referencias,
prefijos importados, undo/redo, revisión y guardado explícito compartido, sin
descargar originales externos.

#### Scenario: Hover independiente del foco de escritura
- **WHEN** el editor conserva el foco y el puntero entra en un bloque, cruza hacia sus controles o se mueve lateralmente por la misma altura dentro del panel
- **THEN** se muestran y conservan los controles de ese bloque, también sobre ambos márgenes; se ocultan al salir de esa franja o panel, y el espacio vacío bajo el párrafo y el foco/escritura por sí solos no los muestran

#### Scenario: Entrada desde un margen y cambio de fila
- **WHEN** el puntero entra directamente por un margen a la altura de un bloque y luego pasa verticalmente a otro bloque, un hueco, un popover u otro panel
- **THEN** aparecen los controles del bloque de esa fila sin hover previo sobre su contenido; cambian al otro bloque o se ocultan en el hueco, popover u otro panel, también después de scroll o resize

#### Scenario: Acceso explícito por teclado
- **WHEN** se invocan acciones del bloque mediante su atajo sin hover
- **THEN** se conserva el menú y el retorno de foco al control; quitar el foco del control permite volver a ocultarlo

#### Scenario: Reordenar y duplicar
- **WHEN** se mueve una lista o un adjunto y se duplica una lista anidada
- **THEN** el movimiento conserva identidades y contenido, la copia recibe identidades únicas y ambas acciones permiten undo/redo

#### Scenario: Buffer protegido
- **WHEN** hay una edición contextual y el guardado falla, entra en conflicto o se intenta salir
- **THEN** permanecen el buffer y las guardias existentes; ningún control contextual omite la revisión esperada ni afirma guardado sin confirmación

#### Scenario: Inserción y cancelación
- **WHEN** se abre el selector de vista o adjunto y se cancela sin insertar
- **THEN** no se añade contenido ni se importa un archivo, y la selección original permanece recuperable

### Requirement: Continuación desde el espacio inferior de Notes

La Note normal SHALL permitir continuar escribiendo mediante un clic primario
sin modificadores en el espacio vacío debajo del último bloque dentro de su
panel. SHALL insertar un párrafo al final y enfocar el cursor dentro de él, o
reutilizar el párrafo final si está vacío sin añadir otro. MUST conservar bloques,
IDs, formato previo, undo/redo y guardado explícito; arrastrar una selección,
usar controles, pulsar dentro de bloques o entre ellos y operar con el editor
deshabilitado MUST NOT insertar un párrafo.

#### Scenario: Continuar después de contenido
- **WHEN** se hace clic en el espacio inferior de una Note que termina en texto, código, lista, encabezado u otro bloque admitido
- **THEN** se añade exactamente un párrafo al final con ID propio y cursor listo para escribir texto sin heredar el tipo del bloque previo; undo/redo y guardar/reabrir conservan contenido e identidades

#### Scenario: Reutilizar un final vacío
- **WHEN** la Note está vacía o ya termina en un párrafo vacío y se pulsa repetidamente en el espacio inferior
- **THEN** se enfoca ese párrafo sin acumular bloques vacíos ni modificar la revisión guardada sólo por enfocarlo

#### Scenario: Respeto del contexto de interacción
- **WHEN** se selecciona texto arrastrando hacia el espacio inferior, se usa clic secundario/modificado, se pulsa un control, un bloque, un hueco intermedio u otro panel, o el editor está deshabilitado
- **THEN** no se inserta contenido por ese gesto; temas claro/oscuro y ancho estrecho conservan la misma regla
