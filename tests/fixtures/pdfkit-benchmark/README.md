# Corpus PDFKit de medición

Fuente sintética propia MIT. Cincuenta páginas con cien líneas ASCII explícitas,
font/posición/leading declarados por el generador PDF propio existente. El golden
se declara antes del parser nativo y usa LF entre líneas. Texto pequeño en puntos
para alojar una carga grande; no es una prueba de legibilidad visual/OCR.
Binarios/manifiesto sólo se generan bajo NOTEKA_TEST_TMP externo, nunca en Git.
La medición contrasta lectura y cada fragmento FTS con este golden independiente.
