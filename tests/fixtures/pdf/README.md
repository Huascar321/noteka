# Corpus sintético PDF

Generador propio `generate.py`, licencia **MIT**, identidad fija
`noteka-pdf-v1-2026-10-13`; no usa azar, fecha, UUID, red, datos personales,
PDFs descargados ni una biblioteca PDF. Los binarios se generan en memoria:
no versionar los archivos PDF de ejecución.

**235 casos** con golden explícito de texto canónico o motivo cerrado. Los
207 anteriores conservan exactamente bytes y hashes; sólo tres motivos se
precisaron al cerrar precedencia: `incremental-free-shadows-old`,
`hybrid-supplement-with-prev` y `hybrid-supplement-free-precedence` ahora
esperan `invalid_pdf` (antes `pdf_unsupported_feature`). Ningún negativo pasó
a positivo; free de un objeto referenciado y Prev=0 son inválidos. La
función `manifest()` calcula desde ese golden descriptores de página, bytes y
SHA-256 de texto/fuente. `golden.sha256` fija tanto el manifiesto serializado
como el generador. El harness comprueba ambos antes de invocar el adapter:
una modificación no puede regenerar expected desde la salida de lopdf.

Reproducción (Python de referencia: zlib runtime **1.2.12**):

```sh
python3 -B tests/fixtures/pdf/generate.py > "$EVIDENCE/corpus-manifest.json"
shasum -a 256 "$EVIDENCE/corpus-manifest.json" tests/fixtures/pdf/generate.py
cargo build --offline -p noteka-pdf-worker --examples
NOTEKA_TEST_TMP="$EVIDENCE" python3 -B scripts/test-pdf-parser.py
cargo build --offline --release -p noteka-pdf-worker --examples
NOTEKA_TEST_TMP="$EVIDENCE" python3 -B scripts/test-pdf-parser.py --profile release
```

`EVIDENCE` debe ser un directorio de pruebas externo al checkout. Un cambio de
zlib puede cambiar bytes comprimidos: revisar su efecto explícitamente, no
actualizar huellas automáticamente ni desactivar la comparación.

## Golden independiente

- Las salidas se escribieron desde los strings sintéticos y reglas de v1,
  **antes** de ejecutar el adapter; no se capturaron con `extract_text`.
- WinAnsi de la fixture: E1→á, E9→é, F1→ñ, 80→€, 93→“. Standard: 27→’,
  60→‘, A1→¡, AE→ﬁ. No hay normalización ni conversión con pérdida.
- `Tj` agrega bytes decodificados; `TJ` concatena strings e ignora números de
  kerning, sin fabricar espacios. Los espacios escritos son contenido real.
- `'`, `"`, `T*` y `Td`/`TD` con desplazamiento vertical agregan un salto sólo
  si ya hay texto y no termina en salto. `ET` hace lo mismo sólo cuando el
  objeto produjo texto. `BT ET` vacío no fabrica texto. `Tm` no reordena.
- Se revisaron los esperados contra estas reglas y los operandos del generador.
  Esto no es una revisión independiente de todo el gate de conformidad PDF.
- El cifrado usa Standard Security Handler R2/RC4 y passwords de fixture,
  incluyendo vacío. No son credenciales reales. El original sigue excluido
  aunque lopdf pueda descifrarlo automáticamente.

## Cobertura y límites

PDF 1.0–1.7, doce variantes Type1, Standard/WinAnsi, cuatro operadores de
show, multibyte/largo, páginas vacía/imagen/mixta, orden de operaciones,
recursos heredados/override, streams partidos, enlaces/acciones pasivos,
cifrado, exclusiones de fonts/mappings/Form/annotations/ActualText, errores
de secuencia/operandos, ciclos/duplicados, nesting, firma/truncado/xref,
objeto comprimido/índice falso, pérdida silenciosa de miembro, checksum,
fronteras/exceso de fuente/texto/páginas/descompresión y bombs xref/objeto.

La continuación agrega revisiones de contenido, trailers/Prev/XRefStm y ciclos,
subsecciones solapadas/filas de longitud errónea, precedencia free/hybrid, objetos
activos solapados y referencia a un miembro ObjStm ausente del xref (reproducida
primero como éxito incorrecto). Recursos/herencia/generaciones y claves de
propiedades de contenido duplicadas/escapadas tienen negativos adicionales.

Codecs lossless: ASCIIHex/ASCII85 (también grupos parciales), RunLength, Flate,
LZW con EarlyChange 0/1, transiciones 9–12 bits y bombs/truncados. TIFF 2 con
bits 1/2/4/8/16, PNG 10–15 y cadena ASCII85→Flate. Los códigos LZW son explícitos,
el generador sólo los empaqueta; un test Rust compara además con weezl ya
transitivo y texto literal independiente. PNG usa el tag de cada fila, incluso
cuando difiere del Predictor declarado, conforme a ISO 32000-1 §7.4.4.4.

La integración agrega framing de revisiones históricas reemplazadas/liberadas,
linearización (incluida revisión incremental), tombstones/free, reemplazo hybrid
y tres revisiones; un índice no puede apuntar dentro de comments/objetos/streams.
ASCIIHex, ASCII85, RunLength, LZW 0/1, TIFF y PNG se prueban también como streams
de carga XRef/ObjStm. `parser_probe` usa bytes crudos; el runtime real usa
`NKPDF001` + bytes. `test-pdf-cli.py` compara los mismos textos/manifest, hashes,
fragmentación y errores por la operación pública real, en ambos perfiles.
Las hints de linearización no se usan para range IO ni certifican visualización.

El harness exige stderr vacío, una respuesta cerrada, igualdad completa de
texto/orden/hash y ningún archivo/core dump. Ejecutarlo dentro de una sandbox
con red denegada verifica además el modo offline de estos casos; no demuestra
una sandbox de privilegios del producto ni aceptación nativa de Home.

El preflight sobre objetos normales y miembros comprimidos ya rechaza claves
duplicadas (incluso nombres escapados) y `endobj` ausente. Se reprodujo primero
la aceptación silenciosa de ambos casos y se agregaron regresiones negativas;
no se cambió el expected para aceptar la recuperación de la biblioteca.
