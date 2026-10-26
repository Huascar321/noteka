# Spec Delta

## ADDED Requirements

### Requirement: OCR de PDF explícito y acotado

El sistema SHALL ofrecer pdf_ocr sólo por enqueue/run explícitos sobre PDF administrado<=1MiB. MUST procesar íntegro el documento o fallar, sin fallback ni alterar originales, con máximo20 páginas/16MP totales,4MP y PNG gris<=1MiB por página y presupuesto compartido5s. El OCR es inferido; no promete transcripción perfecta, orden visual o tablas.

#### Scenario: Escaneo mixto vacío y rotación
- **WHEN** se ejecuta un PDF compatible con páginas escaneadas, textuales, vacías y rotaciones admitidas
- **THEN** cada página física tiene raster/hash y regiones inferidas identificables; las vacías conservan su posición, sin extraer texto por otro perfil ni publicar un prefijo parcial

#### Scenario: Formato recurso o motor no disponible
- **WHEN** la fuente está cifrada/inválida, excede páginas/píxeles/bytes, falta un helper/perfil o vence el presupuesto
- **THEN** falla con motivo acotado, recolecta únicamente sus hijos y conserva originales y generación previa, sin instalar recursos o ampliar límites

#### Scenario: Cancelación pausa y fuente nueva
- **WHEN** cambian pausa/ownership/cancelación/source durante render, OCR o publicación
- **THEN** la cerca impide publicar el resultado viejo; el intento sigue el estado compartido del núcleo y no modifica notas ni datos ajenos

### Requirement: Generación PDF OCR canónica recuperable

El sistema SHALL persistir JSON canónico versionado con página/rotación, raster PNG/hash/tamaño, reconocimiento decimal y motor/OS reales. MUST acotar resultado1MiB/512 chunks UTF-8 y retención compartida32MiB, validar propietarios/particiones/hash/configuración íntegros y publicar una sola generación atómica sin purgar ajenas.

#### Scenario: Unicode inferido y páginas sin regiones
- **WHEN** el resultado incluye texto multibyte, bbox/confianza decimales y páginas vacías
- **THEN** conserva exactamente textos/decimales normalizados y descriptores físicos, sin reemplazar el PDF por PNG o tomar inferencia como original

#### Scenario: Corrupción y cuota compartida
- **WHEN** se altera un chunk/metadato/owner fuera de la página solicitada o se supera retención al reemplazar
- **THEN** lectura/publicación falla completa; no trunca, normaliza datos desconocidos o elimina generaciones ajenas para caber

### Requirement: Lectura paginada PDF OCR sin ejecución

extraction.pdf_ocr_regions SHALL devolver regiones inferidas con página física, ordinal local/global, manifiesto completo y cobertura, por límite1–64/default32 y cursor ligado a workspace/job/generación/límite. MUST revalidar fuente/derivado íntegros sin helper, escritura, indexación o reparación.

#### Scenario: Recorrido y resultado anterior
- **WHEN** se pagina una generación persistida, se reabre/restaura o falla/reintenta su job sin helpers
- **THEN** las regiones aparecen una vez con manifiesto/posición completos y flag de intento previo correcto; cambiar ruta/generación/límite invalida el cursor

#### Scenario: Lector ajeno entrada o fuente inválidos
- **WHEN** se pide con ID/cursor/configuración/tipo ajenos, campos desconocidos o original alterado
- **THEN** se rechaza sin fallback, migración o contenido parcial confiable

### Requirement: Preparación y lector PDF OCR en escritorio

Los paneles SHALL ofrecer pdf_ocr explícito y lectura de20 regiones con texto, página/raster, bbox/confianza y motor/limitaciones completos. MUST preservar acciones/CAS/cupos/lifetimes, editor/buffers/guardias y Home; montar/editar no consulta, renderiza, transcribe o selecciona.

#### Scenario: Acciones separadas y buffers
- **WHEN** se prepara y lee desde una nota pendiente en ambos temas y por teclado
- **THEN** sólo run procesa; lectura no exige selección FTS y muestra inferencia falible sin sustituir el buffer ni descargar originales en base64

#### Scenario: Errores y respuesta tardía
- **WHEN** falla una llamada o cambian ID/tipo/nota/workspace mientras está pendiente
- **THEN** retira evidencia vieja, descarta respuesta tardía y conserva cupo iniciado/guardias sin retries, fallback o reparación automáticos
