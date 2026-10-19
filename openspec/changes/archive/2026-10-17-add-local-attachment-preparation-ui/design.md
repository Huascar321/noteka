# Design

## Context

Ver `proposal.md`. Baseline durable: 424 archivos previos y manifiesto fuera de Git, indicado por `~/.local/share/noteka-verification/active-attachment-preparation-path.txt`. Se inspeccionaron DTOs/dispatcher/guías: no hay attachment.list ni lectura sólo de metadatos; attachment.read descarga base64. `extraction.list/get/control/index` permiten consultar sin originales. `index get` filtrado por job tiene a lo sumo una selección y su cobertura informa compatibilidad. Las llamadas son snapshots independientes; put verifica CAS ante concurrencia.

## Goals / Non-Goals

**Goals:** cerrar este recorte verificable de 5a reutilizando el contrato completo, guardias y almacenamiento existente.

**Non-Goals:** ver proposal; no atribuir soporte por extensión ni deducir encabezados de XLSX. El lector completo se propone después como cambio independiente; ampliar parsers/OCR requiere investigación y aceptación propias.

## Decisions

1. Panel independiente en el cuerpo del editor de notas, sin añadir navegación Home ni integrar acciones en Search. Referencias obtenidas del documento semántico actual (incluido buffer), IDs únicos y etiquetas explícitamente «rótulo de bloque». ID manual sirve a adjuntos sin referencias sin descargar sus bytes. Alternativa descartada: catálogo global ficticio o attachment.read para listar metadatos.
2. Controlador tipado con una cola física compartida de capacidad uno y lifetime por selección; nada al montar ni refrescar la ventana. Consulta de trabajos por adjunto/20 y página reemplazada; seleccionar un job exige inspección explícita. Get job, control get e index get job/1 se secuencian bajo el cupo, sin presentarlos como un snapshot atómico. Cambiar selección limpia datos/cursores/errores. La revisión de la nota nunca se rebasa.
3. Cada botón llama una única mutación, sin automatizar el flujo. Enqueue usa sólo attachment_id/extractor y defaults Rust; run siempre lleva id. Pausa usa revisión del control; cancel/retry revisión del job; put generación observada y revisión del índice. La UI habilita según estado/soporte/cobertura conocidos, pero Rust valida todas las reglas/cuotas. No reset ni purga. Todas las mutaciones invalidan la inspección, muestran recibo de la operación y requieren consulta explícita nueva; ante transporte incierto también, sin retry de escritura. Las lecturas que fallan retiran datos viejos.
4. No bloqueo global del editor/Home por preparar. Desmontar cancela espera/publicación, no la operación Rust iniciada. El texto avisa que salir no cancela; cancelar job es acción diferente. Una mutación todavía en cola se cancela si cambia identidad antes de iniciar. Cupo retenido hasta terminar IPC real. Se prueba con promesas controladas y procesos reales a través del puente.
5. Render nativo escapado, details/pre para configuración/procedencia; no HTML remoto ni almacenamiento de contenido. Mostrar estados literales más explicaciones y límites sin inventar metadata. Los límites son del core, no se elevan ni se implementan como segunda autoridad en TS.
6. El host de desarrollo necesita preparar ambos siblings: hook genérico local PDF/XLSX, manteniendo el script PDF previo. Configuración externalBin incluye XLSX; bundle sigue desactivado. No cambian Rust/IPC ni la distribución/firma.
7. Pruebas del controlador cubren secuencias exactas, CAS, errores/lifetimes/remount. UI headless usa CLI real sobre fixtures sintéticas fuera de Git y snapshots de DB/originales para lecturas y buffers; no es IPC nativo. Regresión previa completa relevante y cuatro gates sin cambios, separados de UI para no competir por recursos. Se preserva cualquier fallo con investigación antes de un nuevo par.

## Risks / Trade-offs

- [Control de pausa global] → nombre y aviso explícitos; reanudar no ejecuta y run puede recuperar jobs vencidos según contrato.
- [Escritura confirmada pero transporte fallido] → invalidar inspección y pedir consulta, nunca repetir automáticamente.
- [Metadatos de snapshots distintos] → etiquetas/cas; no afirmar atomicidad de lecturas compuestas.
- [Adjunto ya sin referencia o referencia pendiente] → mostrar ID y origen del rótulo, sin propiedad exclusiva ni writes de documento.
- [Índice completo costoso aunque filtrado] → conservar validación/caps/gates; ninguna caché de confianza o reset implícito.

## Migration Plan

Sin migración, dependencias ni contrato nuevo (sólo hook/config de helpers del host): v12/API v1/52/MCP 24/44/45 y nueve mutaciones intactos. Revertir sólo la UI no descarta originales/generaciones/selecciones. Contrastar baseline, contratos, Home y corpus/gates antes de sync del único delta; archivar sólo después de evidencia y tareas completas.
