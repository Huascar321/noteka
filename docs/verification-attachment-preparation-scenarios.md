# Matriz de preparación explícita

Cambio `add-local-attachment-preparation-ui`: seis requisitos y trece escenarios.
La evidencia headless usa CLI real, no IPC Tauri. La aceptación global/gates se
registra por separado en [verificación](verification-attachment-preparation-ui.md).

| Escenario del delta | Implementación | Evidencia ejecutable |
| --- | --- | --- |
| Referencias compartidas y selección manual | `attachmentReferences`, selector/ID manual | Cliente referencias únicas/nesting y headless seis IDs/siete bloques e ID manual sin referencia. |
| Apertura y cambio sin efectos | constructor/select sin llamadas; panel fuera de Search | Cliente montar/cambiar no operan; headless conteo inicial, sin búsqueda/base64/reset. |
| Página de trabajos y trabajo elegido | list 20/reemplazo; inspect get/control/index | Cliente cursores/página y secuencia; headless 22 jobs en 20+2, soporte/config/procedencia/revisiones. |
| Incompatible vacío o error | página nula/error separado; texto/raw config | Cliente lectura/error/respuesta ajena; headless vacío, configuración futura, UUID inválido y error hostil escapado. |
| Extracción offline paso a paso | enqueue/run por ID independientes | Headless UTF-8/PDF/XLSX, run pausado, intentos, snapshot sin selección tras extracción y dedupe terminal; helpers reales sin red. |
| Pausa cancelación y reencolado | control CAS/job CAS y recibos | Cliente pausa/run/cancel/retry exactos; headless resume sin ejecución, retry/cancel mantienen generación anterior. |
| Conflicto y resultado incierto | invalidar inspección, needsConsult y error | Cliente CAS/transport sin repetir; headless revisión job/control concurrentes y error HTTP después de cancel real confirmada. |
| Selección compatible sin ejecutar | canSelectGeneration/put con CAS | Cliente generación sin texto/unsupported; headless cuatro puts explícitos (incluido vacío), no-op, índice validado sin run/búsqueda automática. |
| Generación o índice concurrentes | expected_generation_id/expected_revision capturados | Cliente payloads exactos/error; headless otro cliente regenera y CAS de índice con generación intacta. |
| Lectura tardía y cambio de contexto | lifetime/token y key workspace/nota/epoch | Cliente éxito/rechazo tardío e inspección abortada; headless respuesta list tras cambiar ID no publica. |
| Escritura tardía o encolada | pool físico uno y señal comprobada antes de iniciar | Cliente remount conserva cupo y evita mutación todavía en cola; headless cancel tardío tras salir no publica y remount no repite. |
| Temas teclado y texto hostil | controles nativos/Svelte escapado/estilos scoped | Headless Enter en enqueue, foco explícito, rótulos/errores hostiles, 1280/900 ambos temas y cero requests externos; capturas sintéticas inspeccionadas. |
| Buffer y navegación protegida | integración posterior al editor sin writes de nota | Headless título pendiente conservado durante todas las acciones, cancelar y fallo real de guardar conservan panel/buffer; snapshots de entidades/documentos/revisiones/FTS/historial y originales idénticos. |

Los 21 jobs futuros adicionales son una inyección SQL documentada exclusivamente
para paginación/soporte; no son resultados, generaciones ni hits fabricados. Las
fuentes/producto se preparan por CLI con el generador previo sin modificarlo.
Los tests no declaran compatibilidad PDF/Excel genérica, OCR ni aceptación nativa.
