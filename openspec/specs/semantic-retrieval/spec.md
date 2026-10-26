# semantic-retrieval Specification

## Purpose
Recuperar fragmentos locales por significado con perfiles de embeddings reproducibles, generaciones de índice activadas atómicamente y una búsqueda híbrida que degrada a SQL+FTS sin presentar fuentes obsoletas.

## Requirements

### Requirement: Perfil de embeddings reproducible e independiente del LLM

El núcleo SHALL definir un perfil de embeddings versionado con proveedor, runtime, modelo, revisión, dimensión, normalización, tarea, prompts, tokenizer, chunking y cuantización. MUST calcular una huella SHA-256 canónica sobre esos factores y rechazar perfiles incompletos o fuera de los límites publicados. La selección MUST ser independiente de cualquier perfil LLM; cambiar un factor SHALL crear una huella distinta.

#### Scenario: Huella estable y sensible al espacio vectorial
- **WHEN** se valida dos veces el mismo perfil y luego se cambia modelo, revisión, dimensión, normalización, tarea, prompt, tokenizer, chunking o cuantización
- **THEN** las dos primeras huellas son idénticas y cada cambio produce una huella diferente, sin aceptar campos desconocidos ni valores fuera de rango

#### Scenario: Perfil inválido no crea índice
- **WHEN** se solicita construir con dimensión cero, cuantización distinta de f32, proveedor no local o prompt demasiado grande
- **THEN** la operación falla con validación estructurada y no crea generación, chunks ni cambios en la generación activa

### Requirement: Adaptador de embeddings local y explícito

El adaptador SHALL invocar un endpoint HTTP exclusivamente loopback configurado por la solicitud. MUST rechazar esquemas remotos, TLS, hostnames, credenciales, redirecciones y respuestas fuera de límite. MUST aplicar los prefijos query/document del perfil, validar dimensión/finitud/norma y normalizar L2 cuando corresponda.

#### Scenario: Endpoint local responde vectores válidos
- **WHEN** un servidor loopback devuelve la dimensión exacta y valores finitos para los textos enviados
- **THEN** el índice almacena vectores normalizados y registra el endpoint usado, sin enviar contenido fuera de la máquina

#### Scenario: Endpoint remoto, caído o respuesta inválida
- **WHEN** el endpoint no es loopback, no responde, devuelve dimensión distinta, NaN, infinito, vector nulo o un cuerpo demasiado grande
- **THEN** la operación falla con error útil, no publica chunks parciales y conserva intacta la generación activa anterior

### Requirement: Reindexación generacional y activación atómica

El índice semántico SHALL ser derivado y regenerable. Cada construcción MUST escribir una generación `building` invisible a la búsqueda y, tras verificar conteos, texto, dimensión y digest, activarla en una única transacción que retire la anterior. Una falla MUST dejar la activa intacta y la nueva `failed`, sin borrar originales, extracción ni formatos desconocidos.

#### Scenario: Construcción completa activa de forma atómica
- **WHEN** todas las fuentes se validan y todos los chunks se escriben correctamente
- **THEN** una transacción activa la generación nueva, retira la anterior, incrementa la revisión de control y la búsqueda pasa a usar únicamente la huella nueva

#### Scenario: Falla durante construcción o activación
- **WHEN** el proveedor falla, se supera un límite, cambia una fuente o falla la transacción final
- **THEN** la generación nueva queda `failed` sin chunks visibles y la generación activa anterior conserva su huella y resultados

#### Scenario: Concurrencia de construcciones
- **WHEN** ya existe una construcción reciente en estado `building`
- **THEN** una segunda construcción se rechaza como ocupada; una construcción abandonada fuera del plazo puede descartarse explícitamente sin afectar la generación activa

### Requirement: Cobertura, vigencia y fuentes del índice semántico

El índice SHALL conservar por chunk job, adjunto, generación de extracción, hash de fuente, extractor, versión, ordinal, texto, hash y localizador. MUST limitar a 1024 fuentes, 8192 chunks y 8 MiB de texto, rechazando en lugar de truncar. La búsqueda MUST verificar la vigencia y excluir por defecto fuentes obsoletas, informando cuántas excluyó.

#### Scenario: Fuente modificada después de indexar
- **WHEN** cambia el original, la generación de extracción o el contenido indexado de una fuente
- **THEN** sus chunks se marcan obsoletos, no se presentan como vigentes y la respuesta informa la exclusión sin borrar el índice ni el original

#### Scenario: Límites de capacidad
- **WHEN** la selección supera 1024 fuentes, 8192 chunks u 8 MiB de texto, o un fragmento excede 4096 bytes
- **THEN** la construcción falla completa antes de publicar chunks y conserva la generación activa anterior

### Requirement: Búsqueda híbrida con filtros previos, deduplicación y presupuesto

`retrieval.search` SHALL aceptar consultas de 1–1024 bytes y modos híbrido, semántico y léxico, con filtros previos por job, adjunto y extractor. MUST fusionar ambos rankings, deduplicar chunks y respetar un presupuesto de 1–262144 bytes. La respuesta MUST incluir fuentes, localizadores, rangos/puntuaciones, modo y cobertura; el límite SHALL ser 1–50 resultados.

#### Scenario: Fusión y filtros
- **WHEN** una consulta coincide en el índice semántico y en FTS y se filtra por extractor o adjunto
- **THEN** sólo se consideran fuentes que cumplen el filtro, cada chunk aparece una vez y la respuesta conserva ambos rangos/puntuaciones y el localizador original

#### Scenario: Presupuesto agotado
- **WHEN** los mejores chunks superan el presupuesto solicitado
- **THEN** se devuelven sólo los que caben completos, se marca `budget_exhausted` y no se recorta el texto de una cita

### Requirement: Degradación explícita a SQL+FTS

Si no hay generación activa, el runtime falla o la consulta no puede embedderse, `retrieval.search` SHALL degradar a FTS e informar `mode_used` y `degraded`. MUST NOT fabricar resultados semánticos. Si tampoco hay índice léxico, SHALL devolver una página vacía con razones; los errores de validación MUST seguir siendo errores.

#### Scenario: Runtime de embeddings ausente
- **WHEN** el endpoint local no responde y el índice FTS tiene coincidencias
- **THEN** la respuesta usa modo léxico, marca la degradación y devuelve fuentes FTS vigentes con sus localizadores

#### Scenario: Sin ningún índice disponible
- **WHEN** no hay generación activa ni entradas léxicas
- **THEN** la respuesta es vacía, declara ambos motivos y no inventa fuentes ni puntuaciones

### Requirement: Evaluación del backend vectorial y límites medidos

La implementación SHALL documentar en un ADR la evaluación del backend vectorial (persistencia, filtros, cancelación, empaquetado, memoria y métricas). MUST publicar un benchmark reproducible del escaneo exacto y distinguir la latencia del backend de la del modelo. MUST NOT declarar adoptado sqlite-vec, HNSW u otro backend sin verificación local.

#### Scenario: Benchmark del backend
- **WHEN** se ejecuta el benchmark reproducible sobre 1, 4096 y 8192 chunks de 1024 dimensiones
- **THEN** se registran muestras, p50/p95, memoria y versión del binario, y el ADR contrasta el resultado con la alternativa no adoptada sin extrapolar a otros equipos o corpus

#### Scenario: Calidad no inferida de fixtures
- **WHEN** se revisa la evidencia de recuperación
- **THEN** las fixtures sólo acreditan contratos, filtros, activación y degradación; cualquier afirmación de calidad semántica real queda explícitamente pendiente de ejecutar Jina con pesos autorizados
