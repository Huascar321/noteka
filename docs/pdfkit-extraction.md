# Extracción local PDFKit

`pdfkit_text` integra el lector independiente ya archivado mediante un consumidor
Rust acotado. Implementación del [cambio archivado](../openspec/changes/archive/2026-10-20-add-local-pdfkit-extraction/tasks.md); aceptación
conjunta aprobada (8/8); [evidencia](verification-pdfkit-extraction.md). El cierre backend usó SQLite v14/API v1 con 54 operaciones, MCP 26/46/47 según permisos.
La integración posterior de [búsqueda y UI](pdfkit-search-ui.md) usa la barrera v15.
El perfil `pdf_text` estricto sigue disponible y conserva su contrato.

## Uso explícito

Importar un PDF administrado no lo procesa. Encolar `pdfkit_text` tampoco ejecuta
el helper. Habilitar la cola y solicitar `extraction.run` procesa un único claim.
No hay scheduler, fallback entre perfiles o selección FTS implícita.

```sh
./target/debug/noteka --json --workspace "$WORKSPACE" extraction enqueue --input "$ENQUEUE"
# ENQUEUE: {"attachment_id":"UUID real","extractor":"pdfkit_text"}
./target/debug/noteka --json --workspace "$WORKSPACE" extraction run --input "$RUN"
# RUN: {"id":"UUID real del job"}; la cola debe estar habilitada explícitamente.
./target/debug/noteka --json --workspace "$WORKSPACE" extraction pdfkit-pages --input "$READ"
# READ: {"id":"UUID real del job","limit":32,"cursor":null}
```

La lectura pagina páginas físicas completas, incluso `no_text`, con límite 1–64
(default 32). El cursor liga workspace/ruta, operación, job, generación y límite.
Restaurar en otra ruta conserva datos pero no admite cursores del workspace fuente.
`result_from_previous_attempt` distingue un resultado conservado durante retry o
un fallo posterior. Leer/reabrir/restaurar no requiere el helper ni reinterpreta
los chunks JSON como texto. El recorte backend14 rechazaba seleccionar PDFKit en FTS; la integración posterior
permite selección explícita según [búsqueda y UI](pdfkit-search-ui.md).

## Formato, integridad y límites

Document v1 del perfil `apple_pdfkit_text_v1` se conserva una sola vez en JSON
canónico: orden físico, Unicode, saltos, estado, engine/OS/build y limitaciones.
Chunks UTF-8 de hasta 4096 bytes, hasta 512, representación total <=1 MiB. Los
escapes JSON cuentan: 1 MiB de texto nativo no garantiza que el resultado quepa.
No hay truncamiento. Los bytes canónicos cuentan en los 32 MiB compartidos con
UTF-8, PDF estricto, XLSX y OCR; regenerar descuenta sólo la generación de su job.
Snapshots completos siguen limitados a 64 MiB.

Antes de materializar texto, SQL acota cantidades/tamaños y comprueba propietarios,
formatos cruzados y metadatos. Se valida toda representación/hash/partición canónica,
incluido el último chunk. La lectura verifica original/hash/tamaño, no sólo la página
pedida. No se deducen geometría, tablas u OCR ni ausencia visual de `no_text`.
El orden es del parser de selección, sin promesa de orden visual; depende del OS.

El consumidor conserva cercas, cancelación y presupuesto restante de hasta 5 s,
pipes 8 MiB/8 KiB, entorno/CWD temporal y red denegada en macOS. Sólo recolecta su
hijo. El helper conserva CPU 4 s y RSS observado 128 MiB cada 5 ms; no es un límite
duro de RSS/GPU. Plataforma/helper ausente falla explícitamente, sin descarga.

## Migración y recuperación

014 añade dos tablas derivadas y barrera v14 en la misma transacción. Las migraciones
001–013 y la huella de originales base.v2 se conservan. El cliente v13 rechaza v14
sin writes y el cliente14 rechaza v15; no hay downgrade in-place. `workspace.open`
crea un backup consistente previo. Restaurar ese backup con el cliente antiguo en
una ruta nueva recupera v13 exacta. Campos, bloques y configuración futuros no se
normalizan ni descartan al migrar.

Las fixtures reales v13 vacía/mixta y el cliente anterior están preservados fuera
de Git. La continuación del 20/10 verificó 39→41 tablas, filas/SQL/originales,
rollback antes/después de versión y los 53 contratos previos en ambos perfiles.
La aceptación conjunta y los gates se registran por separado; esto no cierra toda 5a.

## Desarrollo y empaquetado

```sh
cargo test --offline -p noteka-pdfkit-runtime --all-targets
cargo test --offline -p noteka-core pdfkit_tests
node scripts/prepare-pdfkit-worker.mjs --tauri
node scripts/prepare-pdfkit-worker.mjs --release --tauri
```

El preparador general incluye este sidecar sólo en macOS. Probes nunca se distribuyen.
El overlay macOS contiene cuatro helpers; el build no-bundle no abre una ventana ni
acredita firma/notarización o IPC nativo. Construir host en target externo evita
sobrescribir los helpers CLI ya fijados para los gates. Home permanece 9/17 intacto.
