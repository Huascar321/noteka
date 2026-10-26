# Fixture PDFKit UI

`generate.swift` dibuja25 páginas sintéticas con CoreGraphics/CoreText offscreen,
sin PDFKit ni ventanas. Declara los textos antes del lector; incluye acentos,
contenido activo tratado como datos y dos páginas vacías. PDF/manifest se guardan
fuera del repositorio bajo NOTEKA_TEST_TMP. No acredita OCR ni tablas.
