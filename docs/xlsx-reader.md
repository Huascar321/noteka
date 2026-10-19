# Lector XLSX acotado (helper independiente)

**Esta guía describe el lector privado y su cierre histórico v10/API v1/51.**
La entrega aislada no habilitó extracción en el producto; la
[integración posterior](xlsx-extraction.md), verificada por separado, añade cola
y lectura CLI/MCP en v11/52. El subconjunto del parser descrito acá se conserva.
No se lanza al abrir, importar, encolar o consultar: sólo run explícito inicia
el helper. No hay modelos, red, cuentas ni ventanas necesarios. FTS XLSX no formó
parte del lector; se añadió después mediante [selección explícita de campos](xlsx-search.md)
en v12, sin modificar el parser ni ejecutar helpers al indexar/buscar.

## Alcance leído

- Paquetes `.xlsx` OOXML **Transitional**, Stored/Deflate, no cifrados; el helper
  recibe bytes, no nombre/ruta. No es un detector de extensiones ni un lector de
  `.xls`, `.xlsb`, `.xlsm`, ODS o OOXML Strict.
- Hojas en orden con ID/nombre/visibilidad; celdas dispersas en orden fila/columna,
  referencias A1 explícitas, dimensión declarada y rango observado separados.
  No se reserva una grilla ni se expanden merges, ni siquiera XFD1048576.
- Ausencia de valor es `cached: null`, distinta de texto vacío. Valores declarados:
  número, texto, booleano, error y fecha ISO. Los números son **strings léxicos**,
  sin f64, redondeo ni locale. `date_iso` conserva la cadena declarada por `t=d`;
  no certifica su calendario. Los seriales numéricos no se convierten a fechas.
- Shared strings, inline y rich text lógico, entidades XML y escapes OOXML
  `_xHHHH_` con pares UTF-16. No hay segunda pasada de unescaping; NUL o surrogate
  suelto se rechazan, no se sustituyen. Espacios se conservan; los finales de línea
  siguen la normalización de XML, no son offsets/bytes del ZIP original.
- Índices de estilo y códigos numéricos declarados, incluidos formatos custom,
  junto con sistema de fechas 1900/1904. No se calcula estilo efectivo/herencia,
  formato visual, moneda, texto presentado ni el falso 29/02/1900 de Excel.
- Fórmula literal y caché separadas. Normal, shared (ancla/si/ref y seguidores)
  y array; shared no se expande. La caché puede faltar o estar vieja: **no se
  recalcula nada**, ni siquiera SUM. Una fórmula WEBSERVICE es texto, no una
  autorización de red. Se rechazan data-table y flags de fórmula no soportados.

No se transcriben imágenes, gráficos, comentarios, formas, formato de rich text,
definiciones de tablas/nombres ni impresión. No se infieren encabezados. Contenido
alternativo, tipos de celdas/extensiones desconocidos y phonetics se rechazan en
lugar de omitir texto que podría cambiar el resultado. No se promete Excel genérico.

## Contenedor y límites

Se validan EOCD/directorio/encabezados/descriptors, nombres/offsets/tamaños,
CRC y consumo completo de Deflate. No se usa `ZipArchive::extract`, no hay
extracción a disco. Nombres ASCII canónicos; no rutas absolutas ZIP, `..`, `%`,
backslash, aliases Unicode ni duplicados. Targets de relaciones pueden ser
absolutos internos (`/xl/...`), pero nunca traversal. Todas las relaciones se
validan, incluidas las no usadas por la transcripción.

Se rechazan ZIP64/multidisco, cifrado, symlinks, macros, conexiones/query tables,
OLE/ActiveX y relaciones externas, aunque sean hyperlinks. El lector no sigue
URLs, entidades externas ni instrucciones. XML es UTF-8, sin DTD/resolver; no hay
fallback con pérdida. Los errores sólo contienen motivos enumerados, nunca
valores/fórmulas, rutas ni objetos del parser.

| Presupuesto | Máximo |
| --- | --- |
| Fuente | 1 MiB |
| Entradas ZIP | 256 |
| Entrada expandida / total expandido | 4 / 16 MiB |
| XML por parte | 100.000 nodos, profundidad 64 de elementos |
| Hojas / celdas totales | 32 / 10.000 |
| Shared strings / estilos / numFmts | 10.000 / 512 / 512 |
| Merges por hoja / nombre de hoja | 1.000 / 31 caracteres |
| Coordenadas (1-based) | 1.048.576 filas, 16.384 columnas |
| Cada texto/fórmula | 131.072 bytes UTF-8 |
| Workbook JSON completo (incluido escaping) | 1 MiB |
| Stdout / stderr permitido al futuro supervisor | 8 MiB / 8 KiB |
| Heap vivo solicitado / CPU / vida | 128 MiB / 4 s / 5 s |

La frontera de un presupuesto no exime de los otros: 10.000 celdas grandes pueden
exceder 1 MiB de representación. No se trunca ni devuelve éxito parcial.

## Protocolo y contención

`noteka-xlsx-protocol` no depende de parser, runtime, SQLite ni UI. Header de 24
bytes little-endian: magic `NKXLS001`, versión u16=1, reservado u16=0, tamaño
fuente u32 (1..1 MiB), presupuesto u32 (1..5.000 ms), reservado u32=0; luego bytes
exactos y EOF. Una sola respuesta JSON estricta con `version`, `parser` y
`outcome` (`success` con hash SHA-256/workbook, o `error` con motivo enumerado).
El validador independiente comprueba versión/parser/hash, límites, tipos,
coordenadas, orden, rangos, estilos, fórmulas y coherencia del workbook.

Runtime compartido con PDF: allocator contabiliza Layout vivos solicitados,
**no RSS/overhead malloc/memoria total del proceso**. Rlimits CPU/core y watchdog
se configuran antes de leer stdin. Acortar presupuesto nunca extiende el inicial;
también termina con stdin/stdout bloqueados o sin padre. Sin core dumps ni logs
de panic. Códigos: 70 setup/IO, 73 heap, 74 panic, 75 deadline; CPU/abort pueden
ser señales del sistema. Error de formato usa exit 0 y outcome error privado.

Esto no equivale a un sandbox FS/red del sistema. El harness se ejecuta con red
denegada y verifica cwd sin escrituras; no demuestra resistencia frente a otro
proceso privilegiado ni contención de kernel. El supervisor futuro deberá añadir
fencing, cancelación, límites de pipes y kill/wait sin retries automáticos.

## Desarrollo y pruebas

Con el toolchain de [desarrollo](development.md), un directorio temporal externo
en `NOTEKA_TEST_TMP` y dependencias previamente disponibles:

```sh
cargo build --offline -p noteka-xlsx-worker -p noteka-xlsx-protocol --bins --examples
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
  python3 -B scripts/test-xlsx-reader.py --profile debug
cargo build --offline --release -p noteka-xlsx-worker -p noteka-xlsx-protocol --bins --examples
sandbox-exec -p '(version 1)(allow default)(deny network*)' \
  python3 -B scripts/test-xlsx-reader.py --profile release
```

Corpus propio en `tests/fixtures/xlsx/generate.py`: ZIP por Python y golden completo
independiente de Rust. Se comparan parser y helper reales, no sólo un mock. No
contiene libros personales; fuentes/expectativas se fijan mediante hashes.
Resultados y brechas se registran en `docs/verification-xlsx-reader.md` al cerrar.
