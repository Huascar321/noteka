# Fixtures OCR sintéticas (MIT)

`generate.swift`: corpus de calidad/errores declarado antes de OCR; textos y
expectativas previos se conservan. `generate-ui.swift`: imagen independiente de
25 líneas para paginación UI, CoreText offscreen, sin ventanas. Los píxeles y
manifest se generan fuera del repositorio con NOTEKA_TEST_TMP.

`blank.png`: PNG propio de1200×300 píxeles blancos RGB8, generado con IHDR, zlib
de filas filtro0 y CRC32; fixture determinista vacía para integración Rust real,
no fotografía ni dato personal. No se usa como prueba de calidad de transcripción.
