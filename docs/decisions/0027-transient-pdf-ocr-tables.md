# ADR0027: tablas como proyección transitoria de PDF OCR

Estado: integración aceptada8/8 con regresión/gates aprobados; sync/archivo
se registra en la [verificación](../verification-pdf-table-reader.md).

La biblioteca independiente `noteka-pdf-table` organiza regiones OCR sin I/O.
Se integra mediante una lectura pública adicional, no un séptimo extractor.
El núcleo valida original/canónico completos, construye la evidencia interna
y proyecta página/rectángulo/columnas/encabezado explícitos bajo snapshot de lectura.
Nunca acepta reconocimiento o hash declarado por el cliente como autoridad.

SQLite16, originales, selección e índice FTS no cambian. La elección geométrica
es transitoria; persistirla requeriría otro cambio con migración/recuperación.
`pdf_ocr` mantiene procedencia de región y sigue siendo el formato fuente.

Las celdas vacías y regiones múltiples se conservan. Encabezado referencia una
fila sin eliminarla; regiones exteriores mantienen ordinales para el lector OCR.
Cobertura de tabla no equivale a devolver toda la generación. El cursor incorpora
hash de todos los criterios además del scope común. Sólo una respuesta completa
puede acreditar `covers_table`; `covers_generation` permanece falso.

Se descarta inferir columnas, dividir texto o sustituir el original por una tabla
corregida. Una región que cruza un borde o filas ambiguas debe producir error,
no una reconstrucción aparentemente exacta. Los límites independientes no aumentan.

La futura selección de motor por plataforma no se disfraza de PDFKit/Vision ni
reescribe generaciones existentes. El usuario pidió el21/10 investigar y comparar
alternativas locales Mac/Windows; el ensayo aislado no constituye soporte Windows,
activación en Noteka ni adopción anticipada. Un nuevo motor debe registrar perfil,
versión/procedencia y sus límites antes de entrar a la cola persistida.
