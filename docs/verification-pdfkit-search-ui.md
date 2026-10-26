# Verificación: PDFKit en índice e interfaz

20/10/2026. Cambio `add-local-pdfkit-search-ui`; aceptación conjunta aprobada. No acredita toda5a, inferencia,
IPC nativo, Home completo o fase6. No se activa ni instala ningún proveedor/modelo. [Revisión](review-pdfkit-search-ui.md).

Evidencia nueva fuera de Git: el puntero `active-phase5-path.txt` bajo
`~/.local/share/noteka-verification/` apunta a `2026-10-20-phase5-140410`.
El checkpoint14 y sus clientes copiados son independientes del código15 probado;
la evidencia del17/10 sigue siendo histórica.

## Recuperación y compatibilidad

La fixture real14 se creó antes de015 con cliente14 durable: cinco generaciones
UTF-8/PDF/XLSX/OCR/PDFKit, cuatro selecciones y41 tablas. Backup/restaurado14
conservan schema, filas y originales exactos. `test-pdfkit-search-migration.py`
verificó en debug/release:

- Cambio sólo de versiones a15;41 tablas/SQL/filas previos exactos, selecciones,
  cursores y hashes íntegros, sin helpers o procesamiento. La cobertura pasa a
  reconocer PDFKit como compatible sin seleccionar; el lector nativo es idéntico.
- Backup14 restaurable con cliente14 en destino nuevo; clientes14 ante15 y15
  ante16 rechazan antes de writes. Rollback BEFORE/AFTER versión conserva14.
- Documento futuro999 y configuración futura con representación léxica propia
  conservados sin normalizar, descartar o ejecutar.
- Catálogos completos comparados:52 operaciones no afectadas exactas; sólo cambia
  input de index e input/output de search. Output index también permanece exacto.
  API54, batch9, errores y permisos/exclusiones MCP conservados.

Resultados: `runtime/pdfkit-search-migration-debug-2/results.json` y equivalente
release-1. El primer intento falló por un assert del harness que esperaba cambiar
ambos outputs; se corrigió el oracle, sin cambiar el output de producción.

## Pruebas y límites de la evidencia

Rust debug/release pasó345 tests por perfil del workspace, excluyendo host desktop.
Los seis tests nuevos cubren Unicode/fragmentación, páginas vacías/metadata no
indexable, selección/no-op/retry, corrupción fuera de filtro/página y fuente
alterada, backup/restore/lotes, reemplazo nativo real e invalidación de cursor.
El máximo admitido de fragmentación se verifica bajo100 páginas/1MiB;512 sigue
como cap defensivo. No se inventa un documento válido de513 fragmentos.

El corpus CLI nativo de22 archivos pasó por cola/parser/lector/selección y búsqueda
en ambos perfiles. El golden MIT preexiste al parser; se contrastan JSON canónico,
páginas físicas/texto, postings y rangos UTF-8/motor/perfil/limitaciones. Lecturas,
selección y búsqueda funcionan con el helper físicamente apartado. Errores de
formato/límites permanecen explícitos y los originales conservan sus hashes.
Logs `pdfkit-search-cli-debug-2` y `pdfkit-search-cli-release-1`. Primer intento
falló por consultar profile en Document en vez del manifiesto público: corrección
del test, preservando el log. El primer compile focalizado usó igualdad sobre un
enum ErrorCode sin PartialEq; el test pasó a matches!, sin ampliar el enum.

Los25 tests del SDK MCP oficial2.3.1 pasaron por perfil, con modern/legacy/auto,
validadores de schemas, paginación real y permisos. PDFKit verifica selección con
write sin run/helper, búsqueda paginada equivalente a CLI, lector previo y datos
de intento anterior. Probes raw stdio pasaron por perfil con red denegada.

## Interfaz headless

El nuevo recorrido PDFKit aprobó822 checks,411 por tema. Fuente propia MIT de25
páginas CoreText sin PDFKit, texto/blank declarados antes del lector. Verifica:
preparación pausada/ejecución explícita, lectura20/5 íntegra con teclado, error real
y error de transporte inyectado identificado, respuesta tardía, buffer y guardias;
selección separada de23 páginas textuales; Search20/3 y rangos contra lector;
locators/motor/limitaciones completos, consulta agrupada/error de cursor parcial,
texto hostil escapado, snapshots read-only exactos y originales intactos.
Cero requests externos. `logs/pdfkit-search-headless-1.log`, runtime
`noteka-headless-GqojKD/`. Pasaron86 tests cliente/editor/puente/controlador conjunto (77+9).

La regresión headless anterior pasó3442 checks (606 base,1248 Search,154
preparación,870 lector,238 agrupada,326 OCR):4264 con los822 nuevos. Home/App/temas y
sus seis artefactos activos permanecen byte a byte iguales al baseline;
`pdfkit-search-home-preservation.json`. Son pruebas de browser sobre CLI real,
no aceptación de IPC/foco/cierre de una ventana Tauri nativa.

## Gates y cierre

Los cuatro gates originales pasaron consecutivamente, con17 binarios fijados
idénticos antes/después, scripts/recetas/cuotas/validaciones intactos:

| Gate | Rango query p95 ms | Lock p95 máximo ms |
| --- | --- | --- |
| Mixto1 | 153,408–174,526 | 12,513 |
| Mixto2 | 156,633–166,651 | 13,532 |
| Original1 | 111,061–119,417 | 17,094 |
| Original2 | 103,608–121,419 | 15,761 |

Targets300/150 sin cambios. `pdfkit-search-gate-decisions.json` y
`pdfkit-search-gates-summary.json` conservan todas las muestras y decisiones.

Dos mediciones adicionales usan fuentes reales propias, golden previo y helpers
apartados al medir, cinco warmups/veinte muestras por caso. Ambas tienen8MiB de
texto total seleccionado, sin sustituir los gates originales:

- Diez PDFs/250 páginas,20.160 bytes nativos y resto UTF-8: query p95 máximo
  128,685ms, lock3,933ms; `runtime/pdfkit-search-bench-4t69lwam/results.json`.
- Ocho PDFs/400 páginas,3.519.600 bytes nativos y resto UTF-8: query p95 máximo
  143,365ms, lock10,405ms; `runtime/pdfkit-search-bench-tpz_nzml/results.json`.

La segunda fixture tiene texto ASCII pequeño en puntos para carga alta; no prueba
legibilidad visual/OCR ni cualquier PDF/caché. Se verifican cada página y cada
fragmento con oracle independiente. No hubo fallo de rendimiento ni optimización
necesaria en este recorte. No se atribuye una garantía universal de latencia.

Clippy -D warnings, fmt, contratos/TypeScript/build y host no-bundle en target
externo15 aprobaron; cuatro helpers presentes y ninguna ventana ejecutada.
Los warnings históricos de tamaño de bundle siguen visibles. La primera
validación staged estricta rechazó un requisito>500 caracteres; se resumió
conservando escenarios y pasó15/15 en la segunda copia. No se relajó el validador.

Cuatro deltas sincronizados/contrastados:6 requisitos añadidos,11 modificados,
53 escenarios delta,39 nombres heredados conservados,115 requisitos no afectados
y nueve capabilities ajenas byte a byte preservados. Main describe sólo esta
integración implementada; toda5a/5b/5c sigue pendiente de sus recortes/dependencias.
El archivo sólo corresponde a este cambio terminado; Home permanece activo9/17.
