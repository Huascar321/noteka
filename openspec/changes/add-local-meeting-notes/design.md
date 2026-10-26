# Design

## Context

El usuario eligió **diseño y modelos primero** el 24/10/2026. Este cambio sólo
tiene planificación; ninguna tarea ejecutable está completada ni debe aplicarse
como parte de esa elección. People lateral se entrega por separado.

NoteType::Meeting y `notes_meetings` ya clasifican notas; no existen captura,
sesiones, tramos, actores ni conclusiones de reuniones. Rust concentra CRUD/CAS,
adjuntos, backup, consultas y el harness; Tauri invoca sus operaciones. SQLite
está en versión 20, documentos admiten 1/2/3 y los adjuntos tienen máximo de 64 MiB.
BlockEditor monta nodos Svelte para vistas incrustadas; ese patrón permite el
componente sin serializar la transcripción en el documento de cada contenedor.

## Goals / Non-Goals

**Goals:** flujo completo `/meet`/detección → captura → transcripción → corrección
de actores → conclusiones revisables; funcionamiento local, recuperación e
integración People/Notes con identidades originales.

**Non-Goals:** bot que entra a la llamada, huellas biométricas de voz, garantía
de nombres reales, sincronización de calendarios, autoenvío de mensajes o tareas,
captura al abrir una nota y paridad total con Notion. Sólo grabar requiere permiso
del SO e inicio explícito; no se añade una confirmación por participante.

## Decisions

1. **Identidad y datos.** Una sesión tiene ID propio y una nota canónica con tipo
   Meeting. Un bloque `meeting` referencia esa sesión desde cualquier nota del
   mismo workspace; quitar la referencia no borra la reunión. Note/Person/Task
   mantienen sus EntityKind. Las notas manuales viven en el documento canónico;
   tramos, actores, correcciones y generaciones se conservan en tablas específicas.
2. **Formato antes de escribir.** Migración aditiva propuesta 021, a reconfirmar
   al aplicar. Documento v4 añade el bloque, conservando 1/2/3 sin reescritura y
   vistas incrustadas en 3/4. Bloque atómico sin texto/hijos y
   `attrs: { meeting_id: UUID | null }`; pendiente guardable con referencia vacía;
   crear/vincular sesión verifica nota, bloque y revisión en transacción. Un
   borrador se resuelve con la guardia habitual antes de iniciar, sin guardar
   silenciosamente otras ediciones. Versiones desconocidas siguen recuperables.
3. **Audio original.** Pistas micrófono/remoto separadas, normalizadas a PCM16
   mono 16 kHz con anclas de reloj y huecos explícitos. Publicación por fragmentos
   administrados inferiores a 64 MiB, manifiesto con hashes/orden/duración y
   journal de la pieza parcial. El core reconoce resultados sólo después del
   audio durable. Backup devuelve busy durante captura/audio original pendiente;
   finalizar crea la barrera. Restaurar recupera estado, nunca dispositivos.
4. **Host y supervisor.** Dominio/validación/estado en noteka-core; proceso nativo
   acotado para captura/inferencia con protocolo privado versionado, mensajes
   limitados, cancelación y fencing. Tauri/MCP mantienen host vivo y la CLI tendrá
   captura en primer plano. El helper macOS se identifica/empaqueta con Noteka
   para que los permisos correspondan al producto, no a un terminal ajeno.
   Un cliente sin host no inicia captura por un proceso
   efímero. Pérdida del propietario cierra dispositivos y deja estado Interrumpida.
   Si cae sin actualizar estado, una lectura identifica la lease vencida sin
   mutar; Recuperar verifica/publica piezas parciales mediante acción explícita.
   Lecturas, selección de pestaña y apertura de workspace no activan workers.
5. **Adaptadores por SO.** macOS: ScreenCaptureKit para audio remoto y AVAudioEngine
   para micrófono. Windows: WASAPI, con loopback por proceso donde la versión lo
   admita y fuente del equipo explícita en la alternativa. Capturar directamente
   evita depender de altavoces. La ruta ASR trata el eco del micrófono con audio
   remoto como referencia; el original se conserva frente al preprocesado derivado.
   El filtro de una aplicación de navegador puede
   incluir otras pestañas: la UI declara el alcance real. Permisos denegados,
   cambio de dispositivo o fuente perdida requieren una decisión visible.
6. **Modelo propuesto.** Nemotron 3.5 ASR Streaming 0.6B + Nemotron 3 Diarization
   mediante NeMo-Speech.cpp, preparado por manifiesto con versión/hash/licencia.
   CPU común, Metal/CUDA/Vulkan según artefacto y hardware; la disponibilidad no
   acredita rendimiento propio. Usar API/SDK de streaming con buffers externos;
   `transcribe --live` del CLI sólo captura micrófono y no resuelve el audio del
   Meet. Evitar su descarga automática de defaults: se pasan modelos locales ya
   preparados. Qwen3-ASR es candidato comparativo si la evaluación de español
   favorece otra solución. [Investigación](../../../docs/meeting-notes-models.md).
7. **Transcripción y actores.** IDs de segmento/hablante estables, offsets de medio
   separados de tiempo de pared, origen mic/remoto y revisiones. Parciales visibles,
   finales durables; salida original inmutable y correcciones como overlay CAS.
   Mostrar Hablante 1/2 hasta asignación explícita; hasta ocho voces automáticas
   en el perfil propuesto. No reciclar etiquetas por fragmento ni deducir persona
   por género/timbre. Asistentes elegidos y presentaciones explícitas en texto
   pueden producir sugerencias con fuente, que requieren confirmar el vínculo.
   Renombrar, vincular a People, agrupar y reasignar tramos;
   solapamientos/atribución dudosa permanecen visibles. Reprocesar crea generación.
8. **Conclusiones.** Perfil LLM independiente, local por defecto. Usa tramos y
   notas manuales con partición según contexto, referencias a IDs/revisión y
   resultado estructurado: resumen, decisiones, preguntas y acciones, sin
   herramientas de escritura habilitadas para el LLM de resumen. Validar
   existencia de fuentes no demuestra verdad semántica; resultado revisable.
   Cambios de transcripción invalidan su vigencia, no borran el resultado anterior.
   Aceptar acciones crea tareas por operación compartida con receipt idempotente.
   Proveedor remoto exige selección para Meeting Notes; ASR/audio siguen locales.
9. **Detección y UI.** Detector configurable sólo con Noteka ejecutándose,
   minimizada incluida. Combina actividad de audio y señales de aplicación/ventana;
   proceso abierto por sí solo no es reunión. Meet en navegador y Zoom/Teams
   nativos son escenarios iniciales; otras apps conservan inicio manual. Popup
   no roba foco ni captura, tiene Iniciar/No ahora y deduplicación/cooldown.
   Señal débil se llama Posible reunión. Falta de señal no impide `/meet`.
   Indicador/fuentes y Pausar/Finalizar quedan accesibles desde componente/control
   flotante. Silencio no finaliza; cierre resuelve captura antes de salir.
10. **Biblioteca y límites.** People integra Meeting Notes y Notes accede a su
    página, sin nueva entrada primaria. This Week usa lunes–domingo civil;
    Calendar, inicio de reunión; By People, vínculos explícitos, incluido Sin
    asignar. Lecturas paginadas 1–100, cursores contextualizados, sin cargar audio
    completo. Primera versión: una captura/host, ocho horas/sesión, 100.000 tramos,
    32 MiB de texto y límite de archivo existente. Alcanzar límite termina de
    forma explícita preservando lo confirmado. Sin cuotas de participantes
    inventadas como confianza del modelo.

## Alternatives

- Reutilizar sólo NoteFields/body: sencillo para notas, pero inadecuado para
  audio incremental, estados, revisiones, recuperación y correcciones concurrentes.
- Un único modelo generativo para todo: mezcla errores de texto/actores/resumen
  y dificulta streaming/edición. Se conservan etapas intercambiables con fuentes.
- Whisper.cpp + diarización separada: alternativa madura para comparar; Qwen3-ASR
  con sherpa-onnx ofrece otra vía común. No instalar todos como dependencia inicial.
- MLX/Core ML exclusivamente: útiles en Mac, no constituyen el runtime Windows.
- Captura web getDisplayMedia: añade selección de pantalla/pestaña por sesión y
  diferencias del WebView; los adaptadores nativos permiten el flujo solicitado.

## Risks / Trade-offs

- Acentos, nombres, doble voz y eco → revisión de calidad con muestras acreditadas;
  VAD y pistas separadas; no prometer WER/DER o nombres perfectos.
- Runtime/SDK reciente y artefactos diferentes → pin de release/capacidades por SO,
  compilación y aceptación separadas; no trasladar benchmarks de otra GPU al Mac.
- Autodetección heurística → posibles falsos avisos/omisiones, deduplicación y
  control manual; no prometer reconocer cualquier reunión en cualquier aplicación.
- Audio persistente ocupa disco → estimación previa (dos pistas ≈230 MB/h), cuota,
  retención elegida y borrar audio como acción explícita con impacto en reproducción.
- Caída/lease vieja/backup → publicación fenced, journal, busy y recuperación sin
  autoencender captura. Cuotas/RSS/latencia se verifican, no se dan por probadas.

## Migration Plan

Antes de DDL: especificar tablas, recuperación, manifiesto de audio, barrera de
backup y conservación de desconocidos. Respaldar un workspace anterior y trabajar
en copia para revisión. Añadir migración desde versión vigente, contratos/catálogos
y documento v4; no convertir notas Meetings importadas sin identidad acreditada.
SQLite/audio originales siguen canónicos; modelos y derivados son reemplazables.
Revertir aplicación no borra tablas/audio: clientes incompatibles rechazan la
versión; restauración sólo desde backup autorizado y sin pisar ediciones posteriores.

## Validation Plan

Compilación y revisión de contratos/preservación; recorrido manual de flujo y
recuperación en copias; aceptación de captura/auriculares/dispositivo/permisos y
calidad separada en Mac/Windows. No se crean ni ejecutan suites en esta propuesta.
Casos concretos y límites están en tasks/specs; una plataforma no ejercitada queda
pendiente. ADR0046 propuesto y [diseño de producto](../../../docs/meeting-notes-design.md).
