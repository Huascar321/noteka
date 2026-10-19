# Revisión del lector OCR independiente

17/10/2026. Se revisaron los seis requisitos/catorce escenarios contra código,
diseño y [matriz](verification-ocr-reader-scenarios.md), con [aceptación técnica](verification-ocr-reader.md).

El adaptador Swift ejecuta el perfil declarado sobre bytes, sin rutas/workspace.
Rust valida la respuesta entera y sus metadatos antes de admitirla; el límite de
os_version precede a split/collect y su caso adversarial está probado. El texto
permanece inferido y escapado como datos; el perfil no corrige ni hace fallback.
Las ocho orientaciones y dimensiones se contrastan con fixtures declaradas antes
del OCR. Los cinco errores de caracteres se conservan, sin ajustar el esperado.

Se comprobaron guardias de entrada/tipo/área, región/texto/representación, campos
desconocidos y salida. El canal de protocolo está separado de diagnósticos nativos;
los ocho flags de instrumentación se rechazan en el binario normal. CPU/core dumps,
watchdog con header real más corto, pipes bloqueados, RSS observado y reap tienen
evidencia ejecutada por perfil. RSS muestreado no se confunde con un límite duro
o el allocator Rust previo; no se inventa contabilidad de GPU.

La comparación contra tar/pins conserva todos los 469 archivos previos. Las únicas
diferencias de código previo son un miembro Cargo y su entrada aditiva en el lock;
las demás actualizaciones son documentación de esta entrega. No hay dependencia
nueva en el core ni cambios de migraciones/contratos/Home. La sonda XLSX conserva
su nombre y comportamiento; OCR usa validate_ocr_reply. No se borran logs o los
baselines de los intentos fallidos.

Los ocho binarios originales y catálogos completos debug/release son idénticos al
baseline después de reproducir los recortes de compilación de los runners. La CLI
debug coincide además con la copia del test SQL reciente. Se documenta qué
evidencia costosa se reutiliza y qué tests son nuevos/repetidos, sin sumar corridas
parciales a las finales ni declarar nuevos gates. La aceptación final usa las
once pruebas actuales del protocolo y 53 casos por perfil.

No quedan defectos materiales demostrados o escenarios sin trazabilidad en este
lector independiente. Se sincronizaron sólo sus seis requisitos añadidos,
preservando los 32 previos, título/Purpose y Home activo 9/17; cambio archivado 6/6. La integración con
cola, persistencia, operaciones, búsqueda/UI y recuperación debe completarse
separadamente; todavía no hay OCR disponible para adjuntos del usuario ni 5a
completa. La meta posterior continúa por dependencias hasta fase 6 inclusive.
