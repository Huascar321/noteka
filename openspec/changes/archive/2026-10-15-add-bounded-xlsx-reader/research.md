# Investigación del lector XLSX

Consulta: **15/10/2026**. Equipo local verificado: Apple M1 Max,
68.719.476.736 bytes (64 GiB), Rust 1.99.0. No se descargaron libros, modelos ni
datos personales; sólo documentación pública y dependencias de desarrollo.

## Parser y precisión

- [Calamine 0.36.1](https://docs.rs/calamine/0.36.1/calamine/enum.DataRef.html), MIT,
  Rust >=1.88, era la estable indicada por crates.io. Se revisaron APIs streaming
  de celdas/fórmulas; `DataRef::Float(f64)` no conserva el lexema numérico original.
  No se incorpora: hacer round-trip por f64 sería incorrecto para la evidencia
  numérica que queremos preservar. No se afirma que calamine sea inseguro.
- [quick-xml](https://docs.rs/quick-xml/0.42.0/quick_xml/) fue considerado como pull
  parser con namespaces. Evita DOM, pero requeriría más estado para relaciones,
  estilos/rich text y validación. No se incorpora.
- [roxmltree 0.21.1](https://docs.rs/roxmltree/0.21.1/roxmltree/), MIT OR Apache-2.0,
  Rust >=1.60, estable indicada por crates.io. DOM read-only, namespaces y
  `ParsingOptions` con DTD desactivado, sin resolver y `nodes_limit` explícito.
  Se limita además tamaño de parte/profundidad/heap/proceso; no se confía en el
  default `u32::MAX` de nodos. No contiene conversión numérica de celdas.
- [zip 9.0.3](https://docs.rs/zip/9.0.3/zip/read/struct.ZipFile.html), MIT,
  Rust >=1.88, estable indicada por crates.io. Sin features default; sólo
  `deflate-flate2` con `flate2/rust_backend`. Preflight antes de `ZipArchive`, CRC,
  tamaños reales y comprobación adicional de consumo exacto de Deflate.
  **Nunca** se usa extract/mangled_name para convertir entradas en rutas de disco.

Versiones/licencias/MSRV verificadas en API oficial crates.io y metadata Cargo;
APIs consultadas también con Context7. Locks exactos para las dos dependencias
nuevas directas. No se modifica la versión de flate2 previamente usada por PDF.

## Formato y semántica

- [Estructura SpreadsheetML](https://learn.microsoft.com/en-us/office/open-xml/spreadsheet/structure-of-a-spreadsheetml-document):
  workbook y hojas separados y ligados por relaciones; no se adivinan rutas por
  el número visible de hoja ni por el orden del directorio ZIP.
- [Fórmulas y valores guardados](https://learn.microsoft.com/en-us/office/open-xml/spreadsheet/working-with-formulas):
  `<f>` es la fórmula y `<v>` su caché; leer uno no recalcula el otro. Noteka
  conserva ausencia explícita, sin fingir ser una aplicación de cálculo.
- [Shared strings/rich text](https://learn.microsoft.com/en-us/office/open-xml/spreadsheet/working-with-the-shared-string-table):
  la tabla es opcional, con índices y runs; se admiten también inline strings.
- [CellFormula](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.spreadsheet.cellformula):
  distinguir normal/shared/array de data-table; metadata compartida no significa
  fórmula expandida. El subconjunto rechaza atributos de fórmula no soportados.

No se copian ejemplos o bibliotecas como golden: el corpus se construye de cero
con resultados semánticos explícitos e independientes en Python. Los documentos
del formato justifican decisiones, no prueban por sí solos la implementación.

## Seguridad y licencias

Se consultó la base pública RustSec en commit
`edb58f5974b83eb8302332821cb6d389777aa351` (14/10/2026):
[RUSTSEC-2025-0168](https://rustsec.org/advisories/RUSTSEC-2025-0168.html) afecta
extract de zip >=1.3/<2.3; declara patched >=2.3. La versión elegida 9.0.3 no está
en ese rango y el lector ni usa extract ni escribe archivos. La consulta del
directorio de roxmltree devolvió 404 (sin entradas en esa revisión), **no una
garantía de ausencia de vulnerabilidades**. No hay `cargo-audit` instalado; esta
revisión documental no se presenta como ejecución de esa herramienta ni como
auditoría exhaustiva de seguridad.

La clausura real y sus licencias se registran durante aceptación a partir de
`cargo tree --target aarch64-apple-darwin` y metadata, separadas del lock completo
multiplataforma. Los tres crates nuevos son MIT y `publish=false`; nada se publica.
La licencia de distribución de la aplicación completa sigue siendo un asunto
separado; añadir un helper no resuelve las declaraciones faltantes del proyecto.
