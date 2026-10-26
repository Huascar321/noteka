# Verificación de integración PDFKit

Cambio `add-local-pdfkit-extraction`, retomado el 20/10/2026. Implementación y aceptación conjunta aprobadas: ocho de ocho tareas verificadas.
Los cuatro deltas están sincronizados/contrastados y el cambio está archivado. No completa toda 5a ni habilita 5b por sí solo.

Evidencia nueva durable: directorio indicado por
`~/.local/share/noteka-verification/active-phase5-path.txt`;
`2026-10-20-phase5-140410`. Baseline inicial de 556 archivos versionados y doce
binarios; checkpoint histórico auditado por SHA-256, sin confundir sus resultados
con ejecuciones nuevas. Los 22 builds históricos permanecen en su evidencia previa.

## Recuperación y contratos

Fixtures v13 auténticas vacía/mixta, copiadas y revalidadas antes de 014: 39 tablas.
Cuatro migraciones reales (dos estados por perfil) comprueban 39→41 tablas,
SQL/filas anteriores, originales, documento/configuración desconocidos,
backup13 restaurable por cliente antiguo, rechazo de v14 por v13 y de v15 por el
actual, sin writes. Triggers BEFORE/AFTER comprueban rollback atómico. Clientes de
migración sin helpers: no hay procesamiento implícito. Informes
`runtime/pdfkit-migration-*/results.json` y `v13-reverified-before-ddl.json`.

Los 53 inputs y outputs anteriores y el error schema son idénticos. Las definiciones
anteriores sólo amplían Operation; tipos nuevos aditivos. Catálogo v1/54, whitelist
batch9 exacta y MCP 26/46/47 con siete exclusiones. `pdfkit-contract-comparison.json`
y catálogos completos v13/v14 fijan la comparación independiente.

## Corpus y pruebas de aplicación

- Corpus PDFKit propio MIT: 22 casos por perfil por cola/CLI reales, 156 llamadas
  CLI por perfil y veinte lecturas con helper ausente. Golden previo al lector,
  fuente y representación canónica independientes, sin falsificar inferencia.
- Migración/backup/restore, corrupción del último chunk/descriptor/owner, Unicode,
  cien páginas físicas, cursores ligados, reintento con resultado anterior,
  cancelación/pausa/fuente alterada y deadline antes de commit.
- Cuota real de cinco formatos: frontera compartida 32 MiB, reemplazo sólo del
  propio job, exceso completo sin purga; publicación OCR real con resultados
  retenidos PDFKit sintéticos identificados como tales.
- 339 tests Rust release aprobados; diez ignores explícitos de mediciones/recursos.
  Debug serial completo también aprobó 339; no se amplió ningún presupuesto.
- 25 pruebas SDK oficial MCP por perfil, con PDFKit nativo, paridad CLI, lectura
  sin helper y permisos separados. 85 pruebas de cliente/editor/puente/lotes.
- Corpus heredados por CLI: 235 PDF y 216 XLSX por perfil, con helpers reales,
  golden independiente y lectura/restauración sin helper.
- UI headless: 606 base, 1.248 Search, 154 preparación, 870 lector, 238 consulta
  agrupada y 326 OCR: 3.442 checks. Ambos temas, cero requests externos. No IPC,
  foco o cierre del host nativo; las ocho tareas de aceptación Home siguen abiertas.

Clippy workspace/all-targets sin desktop pasó con `-D warnings`; el host macOS
no-bundle compiló en target externo y conserva cuatro sidecars por overlay.
`pdfkit-acceptance-binary-pins.json` fija 17 artefactos; todos seguían exactos antes
de medir. No se reconstruye el host entre pins y gates.

## Fallos conservados e investigación

Primeros probes detectaron expectativas de test desactualizadas de catálogo/schema,
fixtures antiguas que debían retirar tablas 014, un golden de backup que comparaba
cursores de rutas distintas, un OS sintético con versión de dos componentes y un
assert de cancelación que esperaba error donde el contrato admite Interrupted.
Se corrigieron los harness, manteniendo fingerprints históricos, contenido,
presupuestos y comportamiento. El primer probe de migración alteró configuración
de una selección existente, invalidando correctamente el índice; la prueba de
configuración futura usa ahora un job separado sin selección. Ningún fallo se
cuenta como aprobación.

Un recorrido OCR inicial carecía de NOTEKA_OCR_UI_SOURCE; falló antes de preparar
la fixture. Se generó la fixture propia de 25 líneas fuera de Git y se repitió sin
cambiar asserts. Un test debug de saturación superó el presupuesto cooperativo
inalterado bajo carga concurrente; el test aislado pasó y se ejecuta además toda
la regresión debug serial. `pdfkit-debug-deadline-review.json` registra condiciones;
no demuestra una causa única ni justifica ampliar límites.

## Trazabilidad

| Requisito delta | Evidencia de aceptación |
| --- | --- |
| Extracción PDFKit persistida sólo por acción explícita | corpus CLI22; fences nativas; ausencia de helper al leer/migrar |
| Páginas nativas canónicas y presupuesto compartido | codec runtime; corrupción SQL; cuota mixta32; backup/snapshots/apply |
| Lectura PDFKit paginada sin procesamiento | 100 páginas/core; CLI con helper ausente; SDK por página; rechazo de FTS |
| Consumidor Rust nativo cercado y descartable | runtime privado siete tests/perfil; cercas del núcleo; deadlines/recolección |
| Evolución recuperable a páginas PDFKit persistidas | fixtures13 reales vacía/mixta, cuatro migraciones y rollback/clientes |
| Contrato compartido de páginas PDFKit | 53 contratos exactos, DTOs estrictos, CLI/catalog/SDK |
| Lectura PDFKit por permisos existentes | SDK26/46/47; writer sin run; siete exclusiones |
| Contrato compartido y versionado; superficie MCP | fingerprints heredados, exhaustive policy, catálogo completo y SDK |

Cuarenta y nueve fuentes protegidas (001–013, lectores privados/corpus/gates) y doce
archivos de Home/App/tema siguen byte a byte iguales al baseline. La nueva migración
no elimina bloques ni propiedades desconocidos. No se instalaron modelos ni se
activaron proveedores/OAuth/pagos/publicación o ventanas nativas.

## Gates y cierre

Cuatro gates originales consecutivos aprobados, con binarios/corpus/gates/cuotas y
validación intactos. Se comprobó el flag de target además del exit code.

| Gate | Rango query p95 ms | Lock máximo p95 ms |
| --- | --- | --- |

| mixed-1 | 152.226–176.788 | 14.027 |
| mixed-2 | 151.967–169.663 | 12.851 |
| original-1 | 108.893–127.583 | 16.197 |
| original-2 | 106.955–125.618 | 16.085 |

Targets invariables: query p95 <=300 ms, lock p95 <=150 ms. Cinco warmups/veinte
muestras por caso; percentil por orden estadístico, no una media de percentiles.
Resultado local del corpus fijado, sin garantía universal. No hubo cambios de
implementación, cuotas, targets o validación para aprobar la cadena. Los fallos
anteriores de test/harness y el deadline debug quedan conservados; la aprobación
posterior no demuestra su causa. Todos los 17 pins binarios siguen exactos.

Revisión de diff/contratos/diseño/escenarios sin divergencias abiertas para este
recorte. OpenSpec estricto 15/15 antes del cierre; no sustituye pruebas de aplicación.

## Sincronización y archivo

Siete requisitos añadidos y dos modificados en cuatro capabilities;107 requisitos
ajenos dentro de esos archivos y las otras nueve capabilities se conservaron.
Diez nombres de escenarios heredados permanecen. Main resultante:183 requisitos
en13 capabilities. Title/Purpose preservados y sin headers de delta en main.
El merge staged y el árbol real aprobaron15/15 items en modo estricto.

Archivado como
[`2026-10-20-add-local-pdfkit-extraction`](../openspec/changes/archive/2026-10-20-add-local-pdfkit-extraction/tasks.md).
`--skip-specs` evita una segunda aplicación: el sync ya estaba hecho y contrastado
por hash. Home continúa activo9/17, sin sincronizar ni archivar. Cerrar este recorte
no cierra5a; no se iniciaron5b/5c ni fase6. Evidencia/cliente14/checkpoint externos
permiten continuar sin reemplazar el baseline histórico ni depender del checkout.
