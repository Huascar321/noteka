# Aceptación conjunta de preparación, lector y búsqueda agrupada

Aceptación técnica del 17/10/2026 para tres recortes de 5a:
`add-local-attachment-preparation-ui` (6 requisitos/13 escenarios),
`add-local-attachment-reader-ui` (4/8) y `add-grouped-local-search` (4/10,
incluidos tres escenarios heredados). Los deltas están sincronizados/contrastados
y los tres cambios archivados separadamente el 17/10. **5a sigue incompleta:** OCR y ampliaciones
PDF/Excel no se presentan como implementados.

## Evidencia durable y alcance

Ruta principal: `~/.local/share/noteka-verification/active-attachment-preparation-path.txt`.
Los otros hitos se localizan por `active-attachment-reader-path.txt` y
`active-unified-search-path.txt`. Los baselines previos a editar tienen 424, 439 y
454 archivos, respectivamente; tar y SHA-256 verificados. El primero conserva
ocho binarios debug/release y catálogos completos. Es continuación v12; no recupera
la comparación histórica exacta v11 perdida, cuya limitación permanece aceptada.

Una regresión **fresca de 36 etapas** se ejecutó sobre Rust/contratos/corpus/gates
inmutables durante la integración de las tres UIs. Preparación aporta la ejecución
compartida; lector y búsqueda agrupada la reutilizan con pins completos y alcance
explícito. No se cuentan como tres regresiones distintas. Se congelaron 222
fuentes actuales; 206 de las 209 iniciales siguen iguales. Cambiaron App, decoder
UTF-8 del puente headless y ayuda de Search, todos revalidados. Los 151 archivos
protegidos, incluidos Rust, contratos, generadores/goldens, gates y Home, están
idénticos al baseline. Ningún archivo original fue descartado.

## Resultados

- **287 tests Rust por perfil**, debug y release; diez ignores intencionales por
  perfil se conservan como tales, no se cuentan aprobados. Fmt/check/Clippy,
  contratos y host all-targets aprobados.
- **216 casos XLSX parser/helper, 216 CLI y 83 de búsqueda por perfil**; procesos,
  proyección independiente y revalidación SQL aprobados. El recorrido debug de
  búsqueda completa tardó aproximadamente una hora; no se redujeron sus 10.000
  celdas, páginas de un item ni validaciones para acelerarlo.
- **235 casos PDF parser y 235 CLI por perfil**, contención, migración/recuperación,
  veinte rondas CLI concurrentes de adjuntos, tres rondas raw MCP y **21 SDK por
  perfil**, aprobados. Los catálogos completos debug/release son idénticos quitando
  únicamente request_id: v12/API v1/52, MCP 24/44/45 y nueve mutaciones batchables.
- **3.116 checks headless de producto:** preparación 154, evidencia previa 1.248,
  regresión anterior 606, lector integrado 870 y búsqueda agrupada 238. Ambos temas,
  CLI/SQLite reales y cero requests externos. Los 848 checks aislados del lector
  son otro hito y no se suman al producto. Fuentes/asserts previos intactos.
- **75 tests de cliente/editor** (19 preparación, 20 lector, 9 agrupada, 14 búsqueda
  y 13 editor/colas/tema) y **11 puente/lotes**, aprobados. Build Svelte/contratos y
  host no-bundle offline/locked con red denegada aprobados sin ejecutar ventanas.
  Clippy native-smoke all-targets también pasó. El warning previo de chunk Vite
  >500 kB permanece visible; no se aumentó su umbral.

## Cuatro gates finales

Ejecuciones release consecutivas, después de terminar la regresión, con los
scripts/corpus/validación/cuotas/targets originales. Query p95 <=300 ms y lock p95
<=150 ms en los cuatro. Rangos entre consultas de cada gate; no promedios globales.

| Gate | Query p95 (ms) | Máximo lock p95 (ms) |
| --- | --- | --- |
| mixed-1 | 166.96–210.00 | 15.30 |
| mixed-2 | 165.19–206.64 | 14.37 |
| unchanged-1 | 119.75–144.91 | 16.35 |
| unchanged-2 | 113.11–123.64 | 19.03 |

Logs completos y muestras: `gate-after-review-stages.log`, `logs/gate-after-review-*.log`,
`regression-stages.log`, `logs/regression-*.log` y `acceptance-summary.json`.
No hay relajación de objetivos ni promesa de 300 ms para cualquier corpus/caché.
El primer gate histórico fallido de la UI anterior y su causa no demostrada se
conservan en su verificación; estos cuatro son ejecuciones nuevas.

Esta continuación también conserva un primer par mixto incompleto: mixed-1
aprobó (query máximo p95 280,07 ms); mixed-2 falló en query-zero con p95
355,936875 ms, máximo 703,456542 ms y cuatro de veinte muestras >300 ms.
Lock aprobó en ambos. `gate-final-stages.log`, `logs/gate-final-*` y
`gate-failure-review.json` preservan ese fallo. Binario/corpus y 222 fuentes eran
idénticos; la ausencia de helpers durante medición es intencional en el gate.
Se observó carga del sistema mayor, sin warning térmico, pero no se demostró
causalidad. Veinte CLI diagnósticos separados dieron p95 externo 200,63 ms;
son instrumentación fuera del gate, no aceptación ni optimización. Tras revisar
muestras/hashes se ejecutó el nuevo par completo y los dos gates UTF-8/PDF,
sin cambiar producción, scripts, corpus, validación, cuotas o objetivos. El fallo
no se borra ni se acepta como excepción.

## Revisión y límites

Revisión contra tar/manifiestos, no contra HEAD vacío. Matrices:
[preparación](verification-attachment-preparation-scenarios.md),
[lector](verification-attachment-reader-scenarios.md) y
[agrupada](verification-grouped-search.md). Las decisiones de UI usan contratos y
lifetime existentes; no hay nueva arquitectura persistente que requiera ADR.
El hook prepara ambos helpers existentes en perfil/target correctos, sin instalar
dependencias/modelos ni publicar. Originales, SQLite, buffers, revisiones, guardias
y validación Rust conservan su autoridad. No hay reintento/rebase o reparación
automáticos.

Los fallos de harness y sus correcciones están preservados en las verificaciones
individuales; no se alteraron esperados del núcleo ni generadores/goldens. El
primer script de comparación de cierre comparó por error un registro JSON con un
digest; se corrigió a su campo sha256, comprobando que ningún protegido difería.

Headless no demuestra IPC/cierre/foco nativo, firma o distribución. **Home sigue
activo 9/17**, con sus ocho tareas de aceptación nativa pendientes y sin cambios
de composición, navegación, clima o secciones. No se sincroniza ni archiva Home.
No se usan archivos personales, pantalla, modelos, proveedores, OAuth o pagos.

## Archivo comprobado

- [Preparación, 9/9](../openspec/changes/archive/2026-10-17-add-local-attachment-preparation-ui/tasks.md): seis requisitos/trece escenarios, local-extraction 22→28.
- [Lector, 6/6](../openspec/changes/archive/2026-10-17-add-local-attachment-reader-ui/tasks.md): cuatro/ocho, local-extraction 28→32 conservando preparación.
- [Agrupada, 6/6](../openspec/changes/archive/2026-10-17-add-grouped-local-search/tasks.md): uno modificado y tres añadidos/diez escenarios; local-text-search 20→23, diecinueve requisitos ajenos y tres escenarios heredados preservados.

Las comparaciones posteriores al sync comprueban cada bloque, título y Purpose,
sin headers de delta en main. Los artefactos completos, incluida .openspec.yaml,
conservan sus hashes tras mover cada directorio. Sólo Home sigue activo 9/17.
