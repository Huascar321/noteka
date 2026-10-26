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
SHALL ofrecer arrastre interno y botones accesibles: mover conserva IDs y
duplicar los renueva también en hijos. MUST preservar atributos, referencias,
prefijos importados, undo/redo, revisión y guardado explícito compartido, sin
descargar originales externos.

#### Scenario: Hover independiente del foco de escritura
- **WHEN** el editor conserva el foco y el puntero entra en un bloque, cruza hacia sus controles o sale hacia otro lugar
- **THEN** se muestran los controles del bloque bajo el puntero, siguen utilizables al cruzar hacia ellos y se ocultan al salir; el espacio vacío bajo el párrafo no cuenta como hover y el foco/escritura por sí solos no los muestran

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
