# Verificación — Home local

Cambio: `add-local-home-dashboard`. Implementación: 12/10/2026; revisión/regresión fresca: 13/10/2026. **Implementado, recorrido integrado headless aprobado y revisión final cerrada; aceptación nativa pendiente. No archivado.** Nueve de diecisiete tareas completamente verificadas; las ocho restantes incluyen pruebas nativas aún pendientes. Código: SQLite v7, documentos v1/v2/v3 y formatos futuros conservados, 38 operaciones compartidas. [Guía](home.md), [ADR 0009](decisions/0009-shared-home-projections.md), [referencia](home-reference.md) y [revisión detallada de 26 escenarios](review-home.md).

## Entorno y comandos

macOS Apple Silicon, M1 Max/64 GiB; Rust 1.99.0, Node 22.22.3, npm 10.9.8, Svelte 5.57.2/TypeScript 6.0.3/Tauri 2.12.3. Lockfiles vigentes. Toolchain temporal indicada en development.md. Fixtures, bases, backups y capturas sintéticos **fuera de Git**.

```sh
cargo test --offline -p noteka-core -p noteka-cli
cargo fmt --all --check
cargo clippy --offline --workspace --all-targets -- -D warnings
cargo clippy --offline -p noteka-desktop --all-targets --features native-smoke,custom-protocol -- -D warnings
npm run contracts:check
npm run test:editor
npm run check
npm run build
npm run tauri build -- --no-bundle
cargo run --offline --release -p noteka-core --example home_benchmark
NOTEKA_TEST_TMP="$TMPDIR" npm run test:headless-bridge
NOTEKA_TEST_TMP="$TMPDIR" npm run test:headless
npx --yes @fission-ai/openspec@1.14.1 validate --all --strict
```

**Cambio de modalidad pedido por el usuario:** no usar su pantalla. `test:headless` usa Playwright CLI 0.1.22, browser headless explícito, sesión efímera sin perfil personal y frontend de producción. El puente de pruebas traduce solicitudes a **CLI Rust y SQLite reales**, con rutas/origen confinados al corpus temporal. No es servidor del producto ni almacenamiento paralelo. Se conservan todas las aserciones DOM anteriores, sin falsear `document.hasFocus()`, más teclado real Playwright. Capturas del browser, no del escritorio. HTTP del contexto limitado a loopback, service workers bloqueados y subprocesses CLI bajo `sandbox-exec (deny network*)` en macOS.

**No equivale a prueba nativa:** IPC y emisión/interceptación de cierre se reemplazan por transporte/eventos de prueba. Los checks heredados con nombre «nativo» verifican aquí la UI/guardia, no NSWindow/WKWebView, permisos/CSP ni cierre de proceso Tauri. `transport: headless-cli-not-native-ipc` identifica el resultado. Reapertura son nuevas cargas del frontend sobre workspace confirmado/restaurado y localStorage del mismo browser, no relanzamientos Tauri. Los fallos/retrasos existen sólo en el puente y feature `native-smoke`, nunca en release. Seis sesiones previstas por ambos temas, deadline 480 s por sesión y waits 12 s.

`test:native` está bloqueado sin `NOTEKA_NATIVE_SCREEN=1`; ese opt-in requiere consentimiento explícito de pantalla y **no se ejecutó después del pedido**. El helper de captura nativa ya autorizado no se vuelve a usar automáticamente.

## Resultados

- Regresión fresca Rust/CLI: **81/81 tests**, incluyendo 10 Home core (8 integración + 2 internos), 2 CLI Home y regresiones anteriores. fmt y ambas configuraciones Clippy: aprobados.
- Adaptador/cola/tema: **13/13**. Puente aislado y guardia de pantalla: **3/3**; origen/ruta/symlink, padre temporal enlazado, operaciones reales, errores y rechazo nativo previo a cualquier build/ventana. También aprobado sin NOTEKA_TEST_TMP con TMPDIR enlazado y con raíz canónica explícita.
- Contratos/check/build: aprobados; Svelte 0 errores/0 warnings. Bundle 507,67 kB, conserva warning orientativo >500 kB, sin elevar umbral.
- Release normal reconstruido el 13/10 tras corregir navegación/captura: `target/release/noteka-desktop`, **Mach-O arm64**; instrumentación Home excluida (marker ausente). Build aprobado sin abrir la app; no firma/notariza/publica.
- Headless integrado fresco: **592 checks compartidos + 14 de teclado/intercalados reales**, todos aprobados en seis sesiones. Por cada tema: main **237**, reopen **25**, restored **34**, más **7** de Playwright (cinco de teclado previos y dos de apertura/captura con transporte retenido y atajo competidor). **Cero solicitudes HTTP a orígenes externos**; metadata identifica HeadlessChrome **154.0.0.0**, service workers bloqueados y CLI sin red. No se eliminaron asserts de las regresiones anteriores.
- Nativo Home: recorrido puntual oscuro previo **19 checks** de layouts/foco/contraste. Último conjunto nativo falló después de **98 checks** en paginación de colección (`Volver al inicio`), antes de completar Home. Fixtures `noteka-native-HXn6Po`; no se declara regresión nativa Home completa. Los 166 checks del cierre anterior de 3c siguen siendo evidencia histórica, no resultados nuevos.

Evidencia fresca fuera de Git: `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-headless-W2Lg5q/`: `results.json`, `network-results.json`, `dark/keyboard-results.json`, `light/keyboard-results.json`, workspaces/restaurados y ocho capturas (tres anchos + Board por tema). Runner cerró su sesión y el puente; no se modificó el perfil personal existente. Evidencia previa de 602 checks y capturas inspeccionadas en `noteka-headless-njfPyQ/`, sin borrarla ni atribuirle los cuatro intercalados nuevos. Ejemplos de `docs/home.md` ejecutados previamente con cuatro respuestas JSON exitosas en `noteka-home-demo.H5rohl/workspace` bajo ese temporal; contratos/CLI no cambiaron en esta revisión.

OpenSpec estricto: **12/12 ítems** aprobados (cambio + once specs), exclusivamente documental. Tarea 4.3 completada con [revisión individual de once requisitos/veintiséis escenarios](review-home.md); 2.1–2.6/3.2/4.2 permanecen abiertas por aceptación nativa. No se sincronizan ni archivan los deltas todavía.

## Trazabilidad de once requisitos / veintiséis escenarios

| Requisito | Escenarios | Evidencia ejecutable |
| --- | --- | --- |
| Consultas Home compartidas y acotadas | Paridad y páginas estables; entrada inválida/cursor ajeno; formato desconocido | `tests/home.rs`: quince vistas con 51 IDs, inputs/cursor/límites, payload/documento futuro y preservación. `cli.rs`: paridad/JSON y escritor concurrente. UI compartida: cada vista/página UI/CLI. |
| Today por vencimiento civil sin alterar My Day | Hoy/vencidas/planificadas; límite civil/zona | `today_workflow_differences_civil_defaults_and_strict_cursors`, `timestamp_and_id_orders_civil_age_and_archived_today`, `deterministic_civil_day_differs_from_utc_without_converting_due_dates`; UI compara Today/My Day y captura sin fechas. |
| Vistas locales Home explícitas | Estado/Inbox; proyectos/clasificación; orden tags/personas | Tests de quince vistas, timestamps/ID/Unicode y workflow Active Projects previo; UI conserva Topic, cinco estados, páginas en las quince. |
| Resúmenes de proyectos desde tareas originales | Resumen completo/CLI; sin tareas/futuro | >50 tareas, compartida entre dos proyectos, vacía/no disponible; snapshot WAL; UI cambio CLI, 0% real versus Sin tareas y estado futuro. |
| Contexto y resúmenes de notas y tags | Conteo global/última empatada/archivo; contexto archivado/futuro | >50 notas, ID estable, archivo/restauración; documentos v1/v2/v3/futuros y referencias archivadas; UI nota/persona/tag y backup/restauración. |
| Grupos Home como referencias de una página | Relaciones múltiples; vacíos/segunda página | Grupos sólo IDs de página, tarea en dos proyectos, Sin proyecto; caps y contexto truncado no limita grupos; UI plegar/paginar/Board/Types. |
| Home local con cobertura visible | Workspace vacío/offline; navegación/pendientes | UI initial: shell ancho/orden/vacío/Goals/clima, nav real; capturas sintéticas y matriz en guía. |
| Presentación Home de vistas originales | Vistas/páginas/teclado; datos no disponibles | UI todas las vistas/páginas, aperturas original/contexto, grupos plegables, 0%/Sin tareas/no disponible/galería sin imágenes. |
| Navegación y captura Home protegidas | Buffer hacia Home; captura proyecto/estado; commit seguido de lectura fallida | UI buffers entidad/colección, cancelar/fallo/conflicto; captura sin commit/relación/estado/clasificación; guardar y continuar seguido de consulta fallida sin repetir creación. Cierre del host nativo pendiente. |
| Lecturas Home vigentes y aisladas | Respuesta tardía; fallo/reintento independiente; refresh tras CLI | `tests/home-reads.mjs`: promesas controladas, cola/desmontaje/día/cursor/retry; UI retraso, fallo aislado, workspace, evento focus y no incremento de revisión. Activación NSWindow integrada pendiente. |
| Temas y foco verificables en Home | Preferencia/buffer; foco/layout | Unit preferencias inválidas/inaccesibles/escritura fallida; UI tema con buffer, reapertura, foco DOM/controles, ratios de tokens, 1920×845/1280×800/900×800 y títulos largos. LocalStorage/foco del host nativo en ambos temas pendientes. |

Snapshot determinista: hook privado de lectura tras adquirir snapshot, otra conexión ejecuta operaciones reales sobre WAL, la respuesta conserva proyecto/tareas anteriores y la próxima ve cambios. Test CLI concurrente lanza treinta updates y sesenta consultas, comparando revisión/estado coherentes. No se presenta stress probabilístico como única prueba de snapshot.

Preservación: antes/después compara referencias/revisiones/payloads/timestamps, documentos crudos, relaciones, historial completo y FTS; las quince vistas original/restaurado comparten resultado y otra ruta rechaza cursor. Recorrido nativo histórico de 3c conserva templates/AST/vistas/adjuntos/documento v3; el recorrido compartido ampliado comprueba esa misma conservación con Home por CLI. No hay migración ni tabla de Home/Goals/clima.

## Medición y comparación visual

Benchmark reproducible en `examples/home_benchmark.rs`: 10.000 entidades (3.000 tasks, 5.000 notes, 500 projects, 500 tags, 1.000 people), 22.000 relaciones, IDs/timestamps deterministas y 5.000 documentos v3 con ~6 KB de cuerpo no seleccionado por Home. Tres warmups y treinta muestras por cada una de quince vistas, límite 50, día 2026-10-12, cache caliente, release aarch64. Mide lector directo y operación con prevalidación/apertura de workspace; bytes de cada página. No mide render/arranque/UI/inferencia ni garantiza rendimiento para corpus arbitrario. Caps y excesos se prueban por separado.

Reporte medido: `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-home-benchmark.json`. Milisegundos, redondeados a dos decimales:

| Vista | Directo p50 | Directo p95 | Operación p50 | Operación p95 | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| tasks_today | 12,02 | 16,83 | 38,54 | 51,23 | 62910 |
| tasks_inbox | 12,48 | 20,12 | 40,45 | 59,68 | 88698 |
| tasks_all | 11,35 | 33,90 | 38,84 | 73,06 | 88745 |
| tasks_done | 11,62 | 17,13 | 38,41 | 62,92 | 88696 |
| notes_all | 16,65 | 23,41 | 48,44 | 84,38 | 41306 |
| notes_inbox | 14,83 | 23,09 | 42,75 | 56,66 | 41310 |
| projects_active | 6,97 | 20,24 | 46,11 | 77,30 | 28203 |
| projects_planned | 5,84 | 14,60 | 33,38 | 70,49 | 26078 |
| projects_board | 5,63 | 9,46 | 32,50 | 52,29 | 28722 |
| tags_recent | 10,73 | 29,07 | 35,46 | 53,15 | 33983 |
| tags_alphabetical | 7,41 | 9,31 | 33,25 | 42,45 | 34014 |
| tags_types | 9,51 | 11,61 | 35,30 | 43,40 | 36284 |
| people_contacts | 2,37 | 3,76 | 28,94 | 36,70 | 20928 |
| people_new | 2,56 | 4,02 | 30,10 | 116,64 | 20908 |
| people_updated | 2,43 | 3,45 | 25,82 | 30,34 | 20916 |

Máximo p95 operación **116,64 ms**, inferior al objetivo orientativo 300 ms en este corpus; no aprobación de usuario ni SLA. SQLite sin WAL: 52.383.744 bytes. No se afirma paridad completa; Goals/clima/fotos/Content/auxiliares siguen pendientes.

Capturas headless inspeccionadas: `noteka-headless-mmwFxe/dark/` (portada 1920×845 y Board 900×800), `noteka-headless-earb3i/light/` (portada 1920×845/900×800) y evidencia final `noteka-headless-njfPyQ/light/` (portada 1280×800 y Board 900×800). Jerarquía, portada azul/verde propia, casa superpuesta, margen ancho, cabeceras azules y tabs siguen la descripción en ambos temas. Títulos largos reﬂuyen y controles conservan nombres/foco. Board tiene overflow **interno** deliberado a 900, no del shell: columnas a la derecha requieren scroll horizontal. «Sin tareas», 0% real y cantidades parciales son visibles, no números de ejemplo. Las capturas superior/Board no muestran simultáneamente las seis secciones; sus órdenes/controles tienen asserts DOM. Portada propia, listas/paginación y galería textual no se presentan como reproducción 1:1; Goals/clima/fotos/Content/auxiliares siguen explícitos en la matriz.

## Revisión y limitaciones

Baseline antes de Home: `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-before-home.tar.gz`, extracción/diffs acotados `noteka-home-baseline-7wnSBY` bajo el mismo directorio temporal. Repositorio sin commit inicial: todo sigue untracked; no commit/push/publicación.

Revisión final completa del alcance acotado (4.3): [informe](review-home.md). Integración de App reutiliza guardias/commit y serializa apertura/captura desde la primera lectura; contratos añaden operación sin cambiar workflows ni migraciones; host instrumentado sólo con feature de prueba. `CollectionPresentation`/`EmbeddedCollectionView` sólo cambian hardcodes CSS a tokens, no consultas ni reglas. Nuevo puente/runner se limita a scripts/tests; aislamiento, temporal canónico y guardia de pantalla probados. Diff persistente/lockfiles/specs principales sin cambios. Aceptación nativa de 2.1–2.6/3.2/4.2 aún pendiente; no se considera completa por el cierre de revisión.

Correcciones durante ejecución:
- Rust: límites SQL convertidos a u32, no usize; macro JSON de benchmark con expresiones parentizadas.
- App: ⌘N desde Home cambia a editor; callbacks Home usan helpers sin guardias reentrantes. Error de apertura visible en Home.
- Retry mantiene cursor/día; restauración de foco sólo cuando se perdió, no roba el foco de otro control.
- Primer intento nativo perdió foco del host al abrir origen desde búsqueda: se reacquiere foco nativo, conserva el assert del editor/localizador. Segundo intento detectó helper ambiguo Notes (cabecera frente a tab); selección ahora se acota a `.home-tabs`, sin alterar reglas de aplicación.
- Headless expuso esperas insuficientes de 150 ms: se espera fin observable de operación, lectura de relación después del commit y foco tras terminar la consulta incrustada. La aserción persisted/restored ahora recorre todas las páginas, no sólo primeros 50 tras el seeding. El lector de tokens del test acepta HEX de tres caracteres (`#fff`) además de seis; la versión previa producía NaN y un falso fallo de contraste claro. Ningún assert anterior se retiró ni se cambió `document.hasFocus()` o el umbral de contraste; estas correcciones aún no se volvieron a ejecutar en host nativo por pedido del usuario.
- Revisión 13/10: se reprodujo apertura Home lenta que admitía ⌘N antes de terminar, con riesgo de limpiar un buffer competidor; `working` ahora protege la primera lectura y la preparación de captura, liberándose antes de enfocar el draft. Dos intercalados por tema pasan con CLI real, retención de transporte y teclado Playwright. El intento anterior falló en el assert nuevo, no se contó como aprobado. También se corrigió el rechazo incorrecto de TMPDIR `/var` frente a `/private/var`; el test adicional conserva los checks de escape/dangling symlink. La cobertura Rust nueva verifica filtros/órdenes que antes sólo tenían evidencia indirecta, 1/3 = 33%, null/futuros y frontera de 10.000 fuentes.

No se inspeccionaron/importaron notas personales de Notion ni descargaron fotos/modelos; sin credenciales/proveedores externos. Sin prueba de lectores de pantalla/teclado físico, notarización, firma de distribución, sincronización o capacidades futuras. Bundle frontend supera aviso orientativo 500 kB por editor/contratos compartidos: no se elevó el umbral ni se afirmó resolverlo.
