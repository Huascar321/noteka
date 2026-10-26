# Verificación del lector XLSX por rango

Cambio `add-local-xlsx-range-reader`, aceptación técnica completa el21/10/2026.
Sincronización/archivo exclusivamente de este cambio tras contraste final. Home9/17 queda independiente.
Evidencia nueva externa: directorio indicado por
`~/.local/share/noteka-verification/active-phase5-path.txt`, cuyo nombre de esta
continuación es `2026-10-21-phase5-191126`.

## Baseline y compatibilidad

Las625 fuentes iniciales coinciden exactamente con el checkpoint16 del20/10;
SHA256 del checkpoint histórico contrastado:
`bf4817addfde64749f6216b8b7cd7e43b5775a5cb6c647cca09c43793e60f728`.
Se preservaron baseline.tar.gz, baseline-pins.json y catorce binarios16 independientes
antes de editar. El reporte histórico13 del17/10 no se usó como estado actual.

`range-contract-comparison.json` verifica55 inputs y55 outputs exactos, errores y
metadata conservados. Adiciones nombradas: una operación/capability descriptor,
variant Operation, raíces read_xlsx_range/xlsx_range y tres definiciones nuevas.
El resto del schema completo queda exacto al retirarlas; hashes históricos no
se modificaron. SQLite16, batch9, siete exclusiones, FTS y canónico XLSX intactos.

## Núcleo y clientes ejecutados

Cinco tests nuevos de Rust verifican scope de cada criterio, cursor falso/obsoleto,
reemplazo transaccional válido, restore en otra ruta, huecos, ausencia, visibilidad,
fronteras A1/64 columnas/XFD1048576, corrupción fuera del rango, original alterado,
pureza scoped y rechazo de batch/campos extra. Fixtures de almacenamiento son
sintéticas y están separadas del corpus parser/CLI real.

`scripts/test-xlsx-range.py` pasó por perfil debug/release:77 XLSX admitidos del
corpus propio MIT,108 hojas,115 páginas y738 llamadas por perfil. Los originales
se crean/extraen usando cliente16 anterior; cliente nuevo sin helper compara
con golden independiente las celdas/ordinals/headers, manifiesto y cobertura,
revisa lecturas previas idénticas y snapshot completo antes/después. Backup nuevo
se restaura con cliente16 previo y conserva documento999 desconocido y originales.

El libro canónico exactamente1MiB con encabezado y límite64 produce el rechazo
correcto response_size_limit; límite1 y lectura sin encabezado funcionan. Se
registran límite2MiB y errores completos, sin subir caps o truncar. Reportes
`runtime/range-cli-debug2/results.json` y `runtime/range-cli-release/results.json`.

SDK oficial2.3.1:28 tests por perfil en modern/legacy/auto, con rangos integrados
en las tres fixtures existentes, paridad CLI, paginación, encabezados tipados y
lectura MCP sin helper. Catálogo28/48/49,56 operaciones y permisos conservados.
La regresión completa release pasó367 tests Rust,13 ignores explícitos; debug
fresco aislado pasó también367 tests/13 ignores. Nuevos tests no sustituyen corpus
nativo/verificaciones históricas de PDFKit/OCR.

## UI y límites de la prueba

90 tests frontend/editor/controladores aprobados. Lector headless pasó1016 checks
(870 anteriores+146 nuevos), dark/light, cero requests externos. Contrasta celdas
con golden independiente, encabezados hostiles, merges sin expansión, páginas20/5,
rango vacío, límites reales2MiB/64 columnas, input no finito que no se convierte
en null, descartes tardíos, buffer/guardias y snapshots por cada llamada. Capturas
headless sintéticas de ambos temas; regresión total4860 checks aprobados.

El puente CLI no acredita IPC/foco/cierre del host nativo. Compilación Tauri
no-bundle se ejecutó sin abrir la aplicación. No se ocupó pantalla, activaron
proveedores, instalaron modelos ni publicaron cambios.

## Fallos preservados y correcciones

- Primer test invocaba un símbolo de scope inexistente; se usó execute_json real.
- Fixture intentaba borrar generación con FK activa y el rechazo batch ocurría
  al deserializar, antes de preview. Se corrigieron los tests: reemplazo válido
  transaccional y rechazo estricto real, conservando FKs/whitelist.
- Assertions de conteo/root schema históricos no excluían la nueva operación.
  Se retiraron sólo adiciones nombradas; los hashes esperados siguen idénticos.
- Primera UI comparaba JSON con distinto orden de claves contra golden Python;
  se compara objeto completo con claves ordenadas, sin omitir ni convertir valores.
- Un primer SDK pidió PNG sintético antes de copiarlo al runtime nuevo. Se
  preservó el fallo y se repitió toda la suite con la fuente presente.
- Primer corpus de rango esperaba éxito para la respuesta de encabezados>2MiB;
  el producto rechazó correctamente. Se añadió comprobación del error y consulta
  explícita con límite1, sin alterar límites ni degradar la representación.

Logs de todos los intentos en logs/ y stages.jsonl. No se atribuyen causas no
probadas de gates históricos; éstos conservan su evidencia previa.

## Gates y cierre técnico

| Gate | Query p95 mínimo–máximo ms | Lock máximo p95 ms |
| --- | --- | --- |
| range-gate-mixed-1 | 158.951209–195.571958 | 14.112916 |
| range-gate-mixed-2 | 164.283708–195.630542 | 13.667167 |
| range-gate-original-1 | 103.803291–117.935750 | 17.116916 |
| range-gate-original-2 | 107.611083–122.275750 | 17.499958 |

Dos pares originales aprobados, corpus/caps/targets intactos300/150ms. Extra de
rango release:41,802375ms p95 máximo, CLI completo, fuente máxima/encabezado/rango
vacío/hoja dispersa, sin helpers,5 warmups/20 muestras. Resultados locales del
corpus declarado, no garantía universal.

Primer debug concurrente falló en una prueba heredada de cuotas por deadline
SQLite. Fuentes productivas de búsqueda intactas; investigación/timeline/condiciones
conservadas en range-debug-deadline-investigation.json. Caso aislado y toda la suite
debug aislada pasan sin cambios (367 PASS); causa única no demostrada. No se
modificaron caps/deadlines/asserts. Primera UI PDFKit carecía de manifest.json de
fixture; se copió el manifiesto sintético conservado y se repitió íntegra (822 PASS).

Host no-bundle, fmt, Clippy -Dwarnings, contratos/check/build y90 frontend aprobados.
4860 headless =606 base+1248 evidencia+154 preparación+1016 lector+238 agrupado+
326 imagen OCR+822 PDFKit+450 PDF OCR. Treinta fuentes protegidas Home/nativos
intactas. De20 binarios fijados,19 permanecen iguales; CLIdebug fue reconstruido
con hash diferente y catálogo completo idéntico. Copia anterior retenida y nueva
comparación en range-binary-comparison.json; no se afirma reproducibilidad binaria.
Release usado en todos los gates permanece idéntico.

Revisión de código/diff y matriz de cinco requisitos/diez escenarios nuevos y
diez requisitos numéricos modificados completa; hashes históricos conservados.
Este cambio no agrega dependencias ni DDL. El proyector PDF independiente propuesto
paralelamente pertenece a otro cambio activo y no se sincroniza con este cierre.


## Cierre OpenSpec

Cuatro deltas staged validados y luego idénticos a main, cinco requisitos nuevos,
diez modificados y29 escenarios heredados conservados; nueve capabilities ajenas
byte-idénticas. Archivado6/6 como2026-10-21-add-local-xlsx-range-reader, con --skip-specs
tras sincronización efectiva. Home permanece activo9/17. El warning no bloqueante
de más de10 deltas se refiere a bloques de requisitos; sólo se tocaron cuatro capabilities.
