# Proposal

Estado: diseño y modelos autorizados el 24/10/2026; implementación pendiente.

## Why

Meeting Notes hoy sólo clasifica notas; no captura audio, transcribe, separa
hablantes ni genera conclusiones. El usuario pide una función propia integrada
con People/Notes, invocable con `/meet` y sugerida al detectar una videollamada.

## What Changes

- Componente de reunión con Notas, Transcripción, Conclusiones y Participantes;
  conclusiones con decisiones, preguntas y acciones revisables enlazadas al audio.
- `/meet` prepara el componente. Un detector local propone iniciar captura al
  reconocer una posible llamada; aceptar Iniciar es el único inicio de grabación.
  No se añade una confirmación por participante. Se respetan permisos del SO,
  selección de fuente, indicador, Pausar/Reanudar/Finalizar y bloqueo de duplicados.
- Captura nativa de micrófono/audio del sistema para macOS y Windows, con alcance
  visible y pistas separadas. Transcripción local provisional y finalización posterior.
- Candidato preferido: Nemotron 3.5 ASR Streaming 0.6B + Nemotron 3 Diarization
  en NeMo-Speech.cpp; español, runtime local común y límite de ocho voces automáticas.
  Qwen3-ASR queda como alternativa comparativa, no dependencia obligatoria adicional.
- Hablantes anónimos estables, renombrables/vinculables a People; corrección de
  tramos y agrupaciones sin afirmar identidad real por el audio ni perder el original.
- Persistencia Rust/SQLite, audio original administrado, generaciones derivadas,
  CAS, recuperación explícita tras interrupción, backup y operaciones compartidas UI/CLI/MCP.
- Biblioteca Meeting Notes bajo People y página propia accesible desde Notes:
  This Week/All/By People/Calendar, con fechas y relaciones reales.

## Capabilities

### New Capabilities

- `meeting-notes`: sesiones, captura autorizada al iniciar, audio, ASR/hablantes,
  correcciones, conclusiones acreditadas, biblioteca y recuperación compartidas.

### Modified Capabilities

- `block-editor`: bloque de reunión persistente y `/meet`, sin perder bloques anteriores.
- `desktop-shell`: biblioteca, componente, popup y estado global de grabación.
- `local-workspace`: backup/recuperación consistentes con audio en captura.

## Impact

Nuevo dominio Rust y migración aditiva propuesta 021 sobre SQLite 20, contratos y
catálogos generados; supervisor de runtime externo acotado, adaptadores nativos de
audio/detección, Svelte/Tiptap y guías. Modelos/binarios/audio fuera de Git. La versión
de migración se confirma al aplicar para no ocupar un número usado por otro cambio.

El usuario decide invocación, popup de inicio y edición posterior de actores.
El runtime/modelos y formatos descritos son propuestas, sin instalación ni mediciones
propias de calidad/rendimiento. Se requiere validar captura real por SO y un perfil
LLM de conclusiones; lo remoto exige selección explícita para Meeting Notes.
No se incluyen un bot participante, sincronización de calendario, huellas de voz,
envío de mensajes, tareas creadas automáticamente ni paridad total con Notion.
