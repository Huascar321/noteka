# Corpus propio PDF OCR

MIT. No contiene documentos descargados ni datos personales. `generate.swift`
produce píxeles con CoreText/CoreGraphics; declara texto esperado antes de OCR.
`generate.py` usa el escritor PDF propio para incrustarlos como imagen sin capa
textual. Produce escaneos, mixed/blank, cajas desplazadas, rotaciones que dejan
texto de costado y rotaciones compensadas con salida vertical,1MiB exacto,
20/21 páginas,16MP exactos/exceso, geometría/PNG/cifrado/framing inválidos.

Los outputs van sólo al directorio externo `NOTEKA_TEST_TMP`; ambos generadores
se ejecutan en la misma carpeta nueva. `--ui` genera otra fuente de13 páginas,
dos vacías y22 regiones esperadas, para paginación/UI en ambos temas. No abre
ventanas. PNG y PDF originales generados se conservan en la evidencia externa.

El límite de calidad se declara sobre el subconjunto limpio, no sobre los casos
de texto de costado. `check-pdf-ocr-quality.py` informa CER/WER exactos contra el
golden, sin normalización. Las pruebas de SQL/protocolo sintéticas se identifican
como tales y no sustituyen este corpus nativo.
