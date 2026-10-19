# Spec Delta

## ADDED Requirements

### Requirement: Extracción PDFKit persistida sólo por acción explícita

El núcleo SHALL admitir pdfkit_text como extractor explícito distinto de pdf_text y ejecutar su helper sólo mediante run sobre un claim válido. MUST preservar pausa, cancelación, revisiones, hash fuente, ownership y cercas de publicación existentes. Importar, encolar, migrar, leer o buscar no ejecuta parsing, no reemplaza un resultado previo ni activa otro perfil por defecto.

#### Scenario: Run explícito y fuente íntegra
- **WHEN** se importa un PDF administrado elegible, se encola pdfkit_text y se ejecuta run con cola habilitada
- **THEN** publica una generación completa ligada a fuente/revisión/configuración admitidas y conserva original, perfil estricto anterior y límites

#### Scenario: Cerca perdida y helper ausente
- **WHEN** se cancela, pausa, pierde ownership, cambia la fuente o falta/falla el helper durante un intento
- **THEN** falla o cancela conforme al contrato sin publicar datos parciales, reemplazar el resultado anterior o ejecutar fallback

### Requirement: Páginas nativas canónicas y presupuesto compartido

El resultado SHALL conservar Document v1 del perfil apple_pdfkit_text_v1 una sola vez en JSON canónico validado y chunks UTF-8 de hasta4096 bytes/512, máximo1MiB. MUST verificar hash, metadatos, páginas y conteos completos, sin offsets originales ficticios. Sus bytes se suman a los32MiB compartidos y snapshots64MiB; un exceso rechaza completo sin purga o truncamiento.

#### Scenario: Unicode y estado de página conservados
- **WHEN** el helper devuelve páginas válidas Unicode y no_text con procedencia real
- **THEN** persistencia y reapertura conservan exactamente representación, orden físico, texto, metadatos y limitaciones, con hash reproducible

#### Scenario: Fronteras y corrupción
- **WHEN** se alcanza o excede una cuota o se altera chunk, JSON, propietario, hash o descriptor
- **THEN** la frontera válida se conserva y el exceso/corrupción se rechaza antes de publicar o leer datos como válidos, sin modificar fuentes o derivados ajenos

### Requirement: Lectura PDFKit paginada sin procesamiento

extraction.pdfkit_pages SHALL leer páginas completas de una generación soportada, con límite1–64/default32, cursor ligado a job/generación/límite, manifest y cobertura explícitos. MUST rechazar cursor/configuración/tipo ajenos y conservar páginas vacías y procedencia. No necesita helper, no regenera ni indexa; este formato no es seleccionable en FTS hasta un cambio independiente.

#### Scenario: Páginas y cursor ligados
- **WHEN** se recorre una generación de hasta100 páginas, incluidas vacías, y se repite tras reapertura
- **THEN** devuelve todas las páginas una sola vez con texto completo, cobertura y cursor correcto sin ejecutar helper o escribir resultados

#### Scenario: Cursor ajeno y selección prematura
- **WHEN** se usa cursor de otro estado/generación/límite, un lector de otro formato o se intenta indexar pdfkit_text
- **THEN** devuelve error tipado sin reinterpretar texto como otro formato, reparar o ejecutar preparación implícita

### Requirement: Consumidor Rust nativo cercado y descartable

El consumidor Rust MUST enviar sólo bytes por NKPDK001, verificar toda respuesta y hash esperado y acotar pipes8MiB/8KiB dentro del presupuesto restante de hasta5s. SHALL conservar errores de cancelación/cerca, usar CWD/entorno temporal y recolectar sólo su hijo al salir o fallar; ningún texto nativo pasa a errores. La plataforma/helper no disponible produce razón explícita sin descargas.

#### Scenario: Respuesta hostil y presupuesto
- **WHEN** el helper entrega respuesta alterada, stderr, exceso, salida no exitosa o bloquea pipes
- **THEN** se rechaza completo dentro del plazo y el proceso propio termina/recolecta, sin confiar sólo en exit0

#### Scenario: Cancelación antes y durante ejecución
- **WHEN** la cerca falla antes de spawn o durante el procesamiento
- **THEN** no inicia o termina/recolecta su hijo y conserva el motivo del núcleo, sin publicación
