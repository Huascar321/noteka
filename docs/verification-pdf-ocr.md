# Verificación PDF OCR local

20/10/2026. Aceptación de `add-local-pdf-ocr`,12/12 verificado, cinco deltas
sincronizados/contrastados y archivo `2026-10-20-add-local-pdf-ocr`.
SQLite16/API v1/55, MCP27/47/48; batch9/exclusiones7. Guía: [PDF OCR](pdf-ocr.md).
Evidencia externa: `~/.local/share/noteka-verification/2026-10-20-phase5-140410/`;
`stages.jsonl`, logs, manifests/fixtures y hashes preservan también los fallos.

- 362 Rust por perfil en la regresión completa (13 ignores explícitos), más
revalidación focalizada por perfil tras el ajuste de enums privados. Clippy
workspace/all-targets `-D warnings`, fmt/check, contratos y build frontend pasan.
- 28 tests con SDK oficial por perfil: modern/legacy/auto, paginación/paridad
CLI, selección/search y lectura sin helpers, permisos y generación anterior.
- 85 pruebas frontend/editor/controladores.4714 checks UI headless:606 base,
1248 evidencia,154 preparación,870 lector,238 agrupada,326 OCR imagen,
822 PDFKit y450 PDF OCR. Temas dark/light; cero requests externos. La última
incluye22 regiones/13 páginas físicas (dos vacías), lectura20/2, Search20/2,
provenance/decimales, errores reales/tardíos, grupos/cursores independientes y
preservación de buffers/guardias/originales. No acredita UI IPC nativa ni Home.
- Host Tauri compilado no-bundle con cinco helpers, sin ejecutarlo. Los20
binarios fijados permanecieron idénticos durante los gates.

Corpus propio nativo21 casos por perfil, golden previo y outputs fuera de Git:
escaneo sin capa textual, mixed/blank, cajas desplazadas, rotaciones normales y
compensadas,1MiB fuente exacto,20/21 páginas,16MP exactos/exceso, geometría,
PNG/cifrado/framing. Probes CPU/RSS/watchdog/pipes/framing y supervisor con
EPERM de red, deadline/cerca/recolección propia. Composición reutiliza Vision
existente. CER1,9886% (7/352 caracteres), WER10,4478% (7/67 palabras) del
subconjunto limpio, sin normalización; no se extrapola a documentos reales.
Texto de costado se registra como limitación fuera de ese subconjunto.

Migración016 usa fixture15 real anterior al DDL, cliente15 independiente y
backup/checkpoint15 (SHA registrado en `checkpoint-v15-pins.json`).41 tablas
previas más dos nuevas vacías; filas/SQL/originales, selección/hashes/cursores y
unknowns exactos. Rollback BEFORE/AFTER versión, old15/future17 sin writes,
backup15 restaurado por cliente15 y backup16 sin helpers.52 operaciones comunes
completas permanecen exactas; index/search amplían sólo los schemas previstos.

Se verifican canónico completo, propietarios/cruces, chunks fuera de página,
paginación/vacíos/intento previo, backup con hash externo reparado corrupto,
preview/apply y retención exacta/exceso32MiB compartida por seis formatos.
Frente512/513 regiones indexables admisible probado sin truncar; Unicode/rangos,
metadata no indexable, selección/CAS y validación fuera de filtro usan Rust.

| Gate intacto | Query p95 máximo ms | Lock p95 máximo ms |
| --- | ---: | ---: |
| Mixto1 |169,196|13,550|
| Mixto2 |172,542|13,864|
| Original1 |125,064|16,342|
| Original2 |125,029|16,090|
| PDF OCR adicional |116.341|4.447|

El adicional usa8MiB totales UTF-8/PDF OCR, 130 páginas
OCR reales y5280 bytes de texto nativo; no son8MiB de OCR.
Cinco warmups/veinte muestras; helpers ausentes al medir, sandbox sin red.
Todos pasan300/150 sin cambiar caps/validación. No es garantía universal.

Fallos conservados y resueltos: fixture PNG ruidoso inicialmente aún admisible;
frontera corregida en otra carpeta, sin elevar límites. Un tamaño sintético de
PNG mal contado; assertions de catálogo/normalización de schemas históricos y
raíces nuevas se corrigieron sin cambiar hashes esperados. Clippy pidió enums
más pequeños: Box privado y serde names explícitos mantienen el wire. Primer
harness UI no alcanzó readiness de Home durante una reconstrucción; la repetición
con binarios estables pasó y la causa exacta no se aisló; un typo del error esperado y env de dos fixtures se corrigieron. El primer
reporte de benchmark OCR reutilizaba `case`: las muestras quedaron guardadas,
se corrigió el nombre y se repitió completo. Ninguno autoriza relajar gates.

Home12 archivos/artefactos y17 archivos nativos PDFKit/OCR protegidos idénticos.
Sin proveedor/modelo/OAuth/pago/publicación. Fase5a sigue incompleta por tablas/
ampliaciones PDF y hojas/encabezados/rangos XLSX;5b/5c no iniciados, fase6 excluida.
