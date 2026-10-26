# Spec Delta

## ADDED Requirements

### Requirement: OCR PDF universal con PaddleOCR

El sistema SHALL ofrecer `paddle_ocr` para extraer texto por página con PDFium + PaddleOCR PP-StructureV3, registrando parser y versión en el formato PDF canónico. MUST validar hash, páginas, texto, deadline y salida; el worker externo MUST usar rutas explícitas de Python/script/modelos y MUST NOT acceder a workspace, SQL, shell o red. `paddle_ocr` SHALL ser opción prioritaria para PDF escaneado configurado; las tablas no se presentan como persistidas.

#### Scenario: OCR universal configurado
- **WHEN** el entorno Paddle está configurado y se ejecuta `paddle_ocr` sobre un PDF
- **THEN** se publican páginas/fragmentos con parser PaddleOCR y el lector PDF los devuelve con página y bytes derivados

#### Scenario: Runtime/modelos ausentes
- **WHEN** faltan Python, script, modelos o el proceso falla
- **THEN** el trabajo falla con `paddle_*` recuperable, no publica parciales y no modifica otros extractores/generaciones

#### Scenario: Límites y procedencia
- **WHEN** el worker devuelve hash/página/texto inválido o supera el límite
- **THEN** el núcleo rechaza la salida completa y conserva la generación anterior
