# Spec Delta

## ADDED Requirements

### Requirement: Estructura tabular Paddle persistida

`paddle_ocr` SHALL conservar por página las tablas de PP-StructureV3 con filas, columnas y celdas (row/column/spans/texto) en JSON ≤1 MiB, con hash, bytes y versión. MUST NOT inventar celdas ni tratar Markdown/HTML como autoridad. La persistencia SHALL ser nullable y aditiva; una estructura inválida MUST fallar completa sin publicar parciales.

#### Scenario: Tabla Paddle con celdas
- **WHEN** PP-StructureV3 devuelve una tabla y se ejecuta `paddle_ocr`
- **THEN** la generación conserva página, filas/columnas, celdas y spans exactos, y el texto de página sigue disponible

#### Scenario: Generación sin estructura o corrupta
- **WHEN** una generación no es Paddle, la estructura supera el límite o el hash no coincide
- **THEN** la lectura/extracción falla de forma recuperable y no modifica originales ni otras generaciones

### Requirement: Lectura acotada de estructura PDF

El núcleo SHALL exponer `extraction.pdf_structure` por job con generación, procedencia y estructura completa, limitada a 1 MiB, sin ejecutar OCR ni modificar datos. MUST distinguir ausencia de estructura de estructura vacía y MUST validar job/generación/hash.

#### Scenario: Lectura de estructura
- **WHEN** se consulta un job Paddle con estructura confirmada
- **THEN** devuelve páginas/tablas/celdas y procedencia, sin ejecutar el worker

#### Scenario: Job sin estructura
- **WHEN** el job no es Paddle o no tiene estructura
- **THEN** devuelve un error de no encontrado/validación explícito, no una tabla vacía inventada
