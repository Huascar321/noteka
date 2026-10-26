# Spec Delta

## MODIFIED Requirements

### Requirement: Lectura PDFKit paginada sin procesamiento

extraction.pdfkit_pages SHALL leer páginas completas de una generación soportada, con límite1–64/default32, cursor ligado a job/generación/límite, manifest y cobertura explícitos. MUST rechazar cursor/configuración/tipo ajenos y conservar páginas vacías y procedencia. No necesita helper, no regenera ni indexa; su selección FTS requiere put y CAS explícitos independientes.

#### Scenario: Páginas y cursor ligados
- **WHEN** se recorre una generación de hasta100 páginas, incluidas vacías, y se repite tras reapertura
- **THEN** devuelve todas las páginas una sola vez con texto completo, cobertura y cursor correcto sin ejecutar helper o escribir resultados

#### Scenario: Cursor ajeno y selección prematura
- **WHEN** se usa cursor de otro estado/generación/límite, un lector de otro formato o se intenta seleccionar pdfkit_text sin CAS vigente
- **THEN** devuelve error tipado sin reinterpretar texto como otro formato, reparar o ejecutar preparación implícita

## ADDED Requirements

### Requirement: Proyección PDFKit por página y bytes derivados

El índice SHALL proyectar sólo texto de páginas PDFKit persistidas, conservando página física, rango UTF-8, tamaño de página, perfil, motor/OS y limitaciones. MUST dividir a4096 bytes sin cortar caracteres ni cruzar páginas y conservar512 fragmentos/1MiB por selección, sin indexar JSON/metadata o iniciar helper.

#### Scenario: Unicode páginas vacías y procedencia
- **WHEN** se selecciona una generación íntegra con Unicode, texto largo y páginas no_text
- **THEN** los rangos consecutivos reconstruyen exactamente cada página textual; vacías no generan fragmentos, pero conservan cobertura/posición en el lector; perfil/motor/limitaciones no se interpretan como texto indexable

#### Scenario: Frontera y exceso sin omisión
- **WHEN** una proyección alcanza la máxima fragmentación permitida por100 páginas/1MiB o su documento excede una cuota
- **THEN** la frontera válida pasa y el exceso se rechaza entero sin truncar páginas, omitir textos o purgar selecciones; el cap512 se conserva aunque resulte inalcanzable bajo esos límites y el lector completo sigue disponible

### Requirement: Integridad PDFKit en índice mixto

Get/search/put MUST validar íntegros JSON/manifiesto PDFKit, hash, proyección, copias y postings, incluso fuera del filtro/página. SHALL incluir metadata completa en la huella y conservar hashes/semántica de formatos anteriores, CAS, invalidación, fuente de candidatos y los límites300/150ms, sin cachés de confianza.

#### Scenario: Corrupción ajena y origen alterado
- **WHEN** cambia texto/metadato/owner/chunk/posting de una selección PDFKit fuera del filtro, o el original de un candidato
- **THEN** la consulta falla completa con razón saneada sin devolver cero hits confiables, reparar, reextraer o ampliar límites

#### Scenario: Selección repetida y reemplazo
- **WHEN** se repite put íntegro con revisión vigente o se reemplaza su generación
- **THEN** no-op conserva filas/revisión; publicar otra generación retira selección/postings previos atómicamente y vence cursor sin seleccionar la nueva, mientras falla/cancel/retry conservan el resultado anterior

### Requirement: Preparación y lectura PDFKit en escritorio

Los paneles SHALL permitir elegir pdfkit_text, preparar y leer páginas por acciones separadas con operaciones/CAS compartidos. MUST presentar una página de20, manifiesto, texto literal, páginas no_text y procedencia/límites, conservar buffers/guardias/lifetimes/cupo físico y no consultar/procesar al montar o editar criterios.

#### Scenario: Flujo explícito y lectura sin selección
- **WHEN** se encola, ejecuta y lee PDFKit desde una nota con buffer pendiente
- **THEN** cada acción tiene recibo propio; sólo run procesa, leer no requiere selección FTS y texto/manifiesto completos no sustituyen el buffer

#### Scenario: Error respuesta tardía y teclado
- **WHEN** fallan fuente/cursor/transporte, se cambia ID/tipo o se navega durante una llamada, usando ambos temas y teclado
- **THEN** se retira evidencia anterior y descarta éxito/error tardío manteniendo cupo iniciado; las guardias y controles se conservan sin interpretar texto activo ni modificar Home
