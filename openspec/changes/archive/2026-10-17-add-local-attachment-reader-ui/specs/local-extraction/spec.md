# Spec Delta

## ADDED Requirements

### Requirement: Lectura explícita de generaciones en escritorio

El escritorio SHALL ofrecer lectura por ID de trabajo y tipo UTF-8/PDF/XLSX elegido explícitamente, sin requerir una selección del índice. MUST llamar sólo al lector compartido correspondiente al solicitar leer, sin importar, procesar, indexar, reparar o descargar originales al montar, editar criterios o abrir detalles.

#### Scenario: Generación sin seleccionar para búsqueda
- **WHEN** se solicita leer una generación existente no seleccionada para Search
- **THEN** el lector comparte la validación Rust y devuelve el derivado con fuente/generación, sin cambiar el índice ni ejecutar helpers

#### Scenario: Abrir editar y refrescar sin consulta
- **WHEN** se abre el panel, cambia el ID/tipo o se activa/refresca la ventana
- **THEN** no se realiza lectura ni procesamiento implícito y se retira cualquier página atribuida a los criterios anteriores

### Requirement: Páginas de lectura con generación y cobertura

El lector de escritorio SHALL presentar una página reemplazable de hasta veinte items, con siguiente/inicio explícitos, generación/procedencia, resultado anterior y covers_generation del núcleo. MUST distinguir página vacía de error y rechazar/desactivar continuación obsoleta sin mezclar generaciones ni afirmar cobertura completa de una página parcial.

#### Scenario: Recorrido completo y vacío
- **WHEN** se recorren más de veinte fragmentos/celdas o una generación sin texto/celdas
- **THEN** las páginas conservan orden/localizadores sin acumulación, cursor nulo termina el recorrido y vacío no se interpreta como OCR o documento visual vacío

#### Scenario: Fuente derivado o cursor inválidos
- **WHEN** la fuente cambia, el derivado es incoherente, el ID/tipo no corresponde o se reemplaza la generación antes de siguiente
- **THEN** el error conserva código/razón y retira texto/cursor anteriores sin reparar, reiniciar ni omitir evidencia automáticamente

### Requirement: Presentación fiel de derivados tipados

El lector SHALL conservar texto canónico y localizadores UTF-8/PDF; PDF muestra su manifiesto de páginas/scope/limitaciones. Para XLSX MUST mostrar hoja/A1/ordinal, tipo/valor léxico, fórmula y caché separados, además del manifiesto de hojas/visibilidad/rangos/merges/estilos/fechas. No infiere encabezados, expande grillas/merges, recalcula ni atribuye offsets del original a texto derivado.

#### Scenario: Texto PDF y UTF-8 literal
- **WHEN** se leen Unicode, espacios/saltos, páginas no_text y fragmentos mayores que el viewport
- **THEN** el texto no se interpreta ni recorta y la procedencia distingue bytes/líneas originales de página/bytes derivados y los límites PDF existentes

#### Scenario: Celdas y metadatos XLSX
- **WHEN** se leen números largos, cadena vacía/ausencia, fórmulas con/sin caché y hojas ocultas/dispersas
- **THEN** se conservan tipos/lexemas/campos completos y metadatos del manifiesto sin redondear, ejecutar, inferir valores o esconder hojas

### Requirement: Lectura de escritorio aislada y sin pérdida de edición

El lector MUST descartar éxito/error tardíos al cambiar ID/tipo/nota/workspace, mantener un cupo físico entre montajes hasta terminar IPC y evitar llamadas todavía en cola al cancelar su espera. SHALL preservar buffers/guardias y ofrecer controles por teclado/foco/texto escapado en ambos temas, sin logs/persistencia de contenido ni remotos.

#### Scenario: Respuesta vieja y remount
- **WHEN** termina una lectura abandonada o se remonta el panel mientras sigue una llamada iniciada
- **THEN** no se publica en el nuevo contexto ni se libera prematuramente el cupo; sólo una lectura física de este panel está activa y una espera cancelada no inicia

#### Scenario: Temas teclado y buffer
- **WHEN** se lee/pagina por teclado con contenido hostil y nota pendiente, o se cancela/falla una navegación protegida
- **THEN** los controles y texto son legibles en ambos temas y viewport estrecho, no se ejecuta contenido y el documento/revisión/buffer quedan conservados
