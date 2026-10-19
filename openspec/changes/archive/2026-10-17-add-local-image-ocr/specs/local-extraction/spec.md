# Spec Delta

## ADDED Requirements

### Requirement: Protocolo OCR decimal estable y compatible

El helper SHALL admitir un protocolo privado v2 explícito con bbox/confianza como cadenas decimales finitas, conservando v1 sin reinterpretarlo. Rust MUST validar, convertir con su parser estándar y canonizar esos escalares sin pérdida del valor f64, sin cambiar features JSON globales, texto reconocido o contratos previos.

#### Scenario: Escalares estables
- **WHEN** se transmiten valores f64 válidos, incluidos extremos/negativo cero y decimales sensibles al parseo JSON
- **THEN** sus valores se conservan al canonizar/guardar/leer repetidamente, mientras NaN, infinito, fuera de rango o cadenas inválidas se rechazan completos

#### Scenario: Versiones privadas separadas
- **WHEN** se llama al lector independiente v1 o al nuevo flujo v2
- **THEN** cada header/respuesta conserva su contrato y cuota; versiones, magic/reservados o campos cruzados se rechazan sin fallback

### Requirement: Extracción OCR administrada explícita

El extractor image_ocr v1 SHALL admitir sólo PNG/JPEG administrados elegibles hasta 1 MiB con configuración v1 predeterminada y perfil local fijo. MUST conservar cola pausada por defecto, deduplicación, cancelación, retry, ejecución explícita por trabajo y generaciones anteriores, sin OCR al importar, abrir, leer o buscar.

#### Scenario: Flujo offline y deduplicación
- **WHEN** se importa una imagen admitida, encola dos veces, reanuda la cola y ejecuta el trabajo elegido con red denegada
- **THEN** se conserva un job deduplicado y se publica su generación sin modificar original/entidades ni seleccionarla para Search

#### Scenario: Fuente o perfil no admitidos
- **WHEN** extensión/MIME/contenido, tamaño, frames, dimensiones o perfil no cumplen el contrato
- **THEN** se informa razón tipada sin reducción, conversión, otro modelo, red, descarga o resultado parcial

#### Scenario: Reintento y resultado anterior
- **WHEN** se cancela/reencola o falla un intento posterior con una generación previa
- **THEN** se conservan la generación anterior y su marca de intento previo hasta una publicación nueva válida, sin ejecución o selección implícitas

### Requirement: Representación OCR persistida e íntegra

La generación OCR SHALL guardar una sola representación canónica v1 de imagen/motor/OS/regiones, con escalares decimales, SHA-256 y chunks UTF-8 <=4 KiB/512, total <=1 MiB. MUST ligar job/generación/fuente, validar identidad/conteos/partición/hash completos y sumar sus bytes al presupuesto retenido compartido de 32 MiB sin duplicar su proyección pública.

#### Scenario: Publicación y lectura fiel
- **WHEN** se publica y reabre una imagen con regiones, Unicode o sin detecciones
- **THEN** se recuperan exactamente texto, valores canónicos, orientación/dimensiones/procedencia y limitaciones; fragment_count cuenta chunks internos, no regiones

#### Scenario: Corrupción y cruce de formato
- **WHEN** hay cambios de hash/chunks/ordinales/owner/manifiesto, bytes incoherentes, campos desconocidos o mezcla de filas UTF-8/PDF/XLSX/OCR
- **THEN** se rechaza la representación completa antes de usarla, sin reparación, inferencia o descarte automático

#### Scenario: Cuota mixta y frontera
- **WHEN** el resultado o la suma retenida con UTF-8/PDF/XLSX alcanza el límite o lo excede
- **THEN** la frontera válida se admite y el exceso rechaza toda publicación, descontando sólo la generación del mismo job reemplazable y sin purgar originales/resultados ajenos

### Requirement: Publicación OCR cercada y contención compartida

El worker MUST ejecutar el helper fuera del lock y verificar fuente, ownership, lease y revisiones de control/job antes y durante publicación atómica. SHALL conservar presupuesto cooperativo 5 s, lease 15 s, límites de pipes y contención nativa declarada; pérdida de fence/deadline mata y recolecta al hijo, sin publicar datos parciales.

#### Scenario: Control cancelación y otro worker
- **WHEN** cambia control/job/ownership, vence lease o otro cliente cancela mientras corre OCR
- **THEN** el intento viejo no publica y el estado recuperable sigue las reglas existentes, sin sobrescribir el trabajo confirmado por otro cliente

#### Scenario: Fuente cambia al procesar
- **WHEN** el original o su metadata/hash cambia después del claim o antes del commit
- **THEN** el intento falla con motivo explícito y conserva originales, resultado previo e índice confirmado

#### Scenario: Helper ausente o incorrecto
- **WHEN** el helper falta, se bloquea, excede recursos/pipes o responde con protocolo/perfil/hash inválidos
- **THEN** run falla de forma recuperable y recolecta el proceso; describir, leer o restaurar resultados previos no necesita el helper ni hace fallback

### Requirement: Lectura OCR completa por páginas

extraction.ocr_regions SHALL leer 1–64 regiones (32 por defecto), con manifiesto, generación/fuente, intento previo, ordinal y bbox/confianza decimales del marco orientado, en <=2 MiB. MUST revalidar original y representación completos antes de paginar, ligar cursor a scope/job/generación/operación/límite y no ejecutar OCR o indexación.

#### Scenario: Páginas y vacío
- **WHEN** se recorren más de veinte regiones o una generación sin detecciones
- **THEN** se conservan orden, texto y campos completos, siguiente termina en null y covers_generation describe sólo la página; vacío no demuestra ausencia de texto visual

#### Scenario: Cursor fuente o derivado inválidos
- **WHEN** cambia la generación/fuente, el cursor es ajeno o el tipo/resultado es incompatible
- **THEN** se devuelve error tipado sin mezclar generaciones, reiniciar, transcribir o reparar automáticamente

### Requirement: Proyección OCR explícita para búsqueda

El índice SHALL seleccionar OCR sólo con put explícito y CAS de generación/revisión, proyectando texto reconocido por región en fragmentos UTF-8 <=4 KiB, hasta 512/1 MiB por generación. MUST conservar bbox/confianza, proyección y rango de texto reconocido sin inventar bytes de imagen, filtrar por confianza o indexar JSON/metadata; las cuotas globales previas se conservan.

#### Scenario: Selección búsqueda y cero texto
- **WHEN** se selecciona una generación OCR íntegra, incluida una sin detecciones
- **THEN** sólo su texto reconocido entra en FTS con localizador de región y cobertura explícitos; extraer/leer por sí solo no incorpora hits

#### Scenario: CAS y regeneración
- **WHEN** otro cliente modifica índice/generación entre consulta y put, o se regenera una selección vigente
- **THEN** CAS obsoleto se rechaza; una publicación nueva invalida la selección anterior sin seleccionar otra ni repetir escrituras

#### Scenario: Proyección excesiva
- **WHEN** una representación admitida produce más fragmentos/bytes de los permitidos para indexarla
- **THEN** el put completo se rechaza y conserva generación/índice anteriores, sin cortar regiones, omitir evidencia o relajar cuotas

### Requirement: Verificación OCR íntegra en índice y consulta

Índice/búsqueda MUST validar manifiesto, representación, proyección, copias y postings completos de las selecciones OCR, incluyendo toda su metadata en la huella. SHALL conservar validación de candidatos fuente, cobertura y rechazo completo de corrupción/futuros no admitidos, sin cambiar hashes o semántica UTF-8/PDF/XLSX ni los gates fijados.

#### Scenario: Derivado o postings alterados
- **WHEN** se altera bbox/confianza/texto, su huella, copias, postings, conteos o representación de una selección
- **THEN** get/search/put rechazan el estado inválido sin éxito parcial, caché que omita validación, reset o reconstrucción automáticos

#### Scenario: Consulta mixta y fuentes
- **WHEN** se consulta un índice mixto o se altera el original de un candidato
- **THEN** se conservan filtros/cursores/cobertura y verificación fuente del contrato existente; los corpus/gates previos mantienen query p95 <=300 ms y lock p95 <=150 ms

### Requirement: Preparación y lectura OCR explícitas en escritorio

La interfaz SHALL permitir elegir image_ocr y leer sus regiones por acciones explícitas independientes de Search, mostrando perfil/procedencia, inferencia, revisiones, límites y errores reales. MUST conservar páginas de veinte, texto/manifiesto completos, sin consultas/procesamiento al montar/editar/activar el lector ni reinterpretar un tipo nuevo como XLSX.

#### Scenario: Preparar seleccionar y leer
- **WHEN** se prepara una imagen administrada y se solicita leer su generación desde una nota
- **THEN** cada paso usa su operación/CAS y recibo, el lector muestra regiones/metadata sin selección obligatoria y ningún paso posterior se ejecuta solo

#### Scenario: Presentación inferida y límites
- **WHEN** se muestran regiones, error, resultado previo o cero detecciones
- **THEN** bbox/confianza y rangos se identifican correctamente, confianza no es certeza y complete/covers_generation no prometen transcripción de todos los caracteres

### Requirement: Lifetime y edición protegidos durante OCR

Los paneles MUST descartar éxito/error tardíos y esperas canceladas al cambiar contexto, sin liberar cupos de IPC iniciado. SHALL conservar buffers/guardias, teclado y ambos temas, sin guardar notas desde OCR, registrar contenido persistente en el navegador o modificar composición/navegación Home.

#### Scenario: Respuesta vieja y cambio de nota
- **WHEN** termina una lectura/mutación OCR abandonada o se remonta el panel con IPC activo
- **THEN** no publica datos/errores en el nuevo contexto, no repite la acción y mantiene el cupo físico hasta terminar la llamada iniciada

#### Scenario: Buffer temas y teclado
- **WHEN** se extrae/lee/busca con contenido hostil y una nota pendiente, o se cancela/falla la navegación protegida
- **THEN** el contenido se escapa, los controles funcionan en ambos temas/viewport estrecho y buffer/revisión/guardias permanecen intactos

### Requirement: Configuraciones OCR futuras conservadas

El sistema MUST conservar crudos los jobs/configuraciones no soportados sin ejecutarlos, convertirlos, repararlos o borrar resultados. SHALL distinguir soporte del contrato de disponibilidad real del helper/perfil y mantener las operaciones antiguas offline sin instalar recursos para resolver indisponibilidad OCR.

#### Scenario: Metadata futura sin ejecución
- **WHEN** se consulta un job con configuración/extractor futuro o se intenta ejecutarlo/indexarlo
- **THEN** sus datos se conservan, se identifica incompatibilidad y ejecución/selección se rechazan sin reinterpretación o purga

#### Scenario: Núcleo sin motor OCR disponible
- **WHEN** el helper/perfil no está disponible y se usan notas, lectores existentes o resultados OCR persistidos
- **THEN** esas operaciones siguen independientes del motor; sólo una ejecución OCR reclamada falla con razón explícita y sin activar alternativas
