# Proposal

## Why

5a aún no transcribe imágenes. La investigación local encontró Apple Vision revision 3 en macOS, con perfil fast español/inglés que reconoció dos líneas sintéticas sin red ni instalaciones. No demuestra calidad general, integración persistente o soporte PDF. Este cambio acotado verifica primero un lector OCR descartable y su contrato Rust.

## What Changes

- Helper macOS explícito para PNG/JPEG de una imagen, hasta 1 MiB y cuatro millones de píxeles, con orientación declarada y salida por regiones normalizadas.
- Perfil fijo revision 3/fast/es-ES/en-US, sin corrección, detección automática de idioma, fallback accurate, descarga ni proveedor externo. Si el sistema no dispone del perfil, falla explícitamente.
- Protocolo privado validado en Rust: hash de fuente, dimensiones/orientación, motor/OS, texto reconocido, confianza y regiones; cuotas sin truncamiento. El texto se identifica como inferido, no literal ni prueba de ausencia cuando no se reconoce nada.
- Contención del proceso: CPU cuatro segundos, watchdog cinco desde inicio del helper, presupuesto que sólo se acorta, pipes/salida acotados, core dumps deshabilitados y salida fija sin contenido en errores. Rechazo si RSS máximo observado supera 128 MiB; se documenta la diferencia con el allocator Rust de PDF/XLSX, que permanece intacto.
- Corpus sintético propio, validación adversarial Rust, pruebas reales con red denegada, procesos/errores/recursos y mediciones de calidad separadas de integridad.

## Non-goals

No modificar cola, SQLite v12, API v1/52, MCP, índices, editor, Home o hooks Tauri; no seleccionar o procesar adjuntos del usuario. No PDF escaneado, búsqueda OCR, estructura de tablas, escritura manuscrita, descripción visual, exactitud perfecta o soporte de otros sistemas. La integración con persistencia/recuperación y búsqueda requiere el cambio siguiente. No relajar validaciones o cuotas existentes.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `local-extraction`: lector OCR local independiente acotado con contrato y límites explícitos, previo a integrarlo en operaciones compartidas.

## Impact

Nuevo protocolo Rust, helper Swift sobre frameworks ya presentes, preparador offline y fixtures/pruebas. Decisión de implementación propuesta: Apple Vision fast como backend macOS inicial, documentada en ADR. No se redistribuyen frameworks o pesos. Requiere SDK/toolchain macOS ya instalado; disponibilidad/errores reales se verifican, sin activar un LLM. Baseline durable de 469 archivos y ocho binarios fuera de Git. La meta continúa hasta fase 6 por dependencias, con intervenciones reales de acceso separadas; este recorte no cierra 5a.
