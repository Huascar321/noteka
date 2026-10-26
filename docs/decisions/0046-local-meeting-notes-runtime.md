# ADR0046: reuniones locales con etapas y originales conservados

Fecha: 24/10/2026. **Estado: propuesto; pendiente de implementación y aceptación.**

## Contexto

El usuario pidió People con su referencia y una función específica Meeting Notes,
con `/meet`, sugerencia al entrar a una llamada, transcripción, actores editables
y conclusiones. Eligió primero diseño/modelos. El tipo de nota Meeting existente
no implementa audio. SQLite 20 y documentos v1/v2/v3 son el punto de partida.

## Decisión propuesta

Separar captura nativa, ASR, diarización y LLM. Rust concentra sesión/revisiones,
validación y publicaciones; SQLite y audio original administrado son canónicos.
Un proceso supervisado usa adaptadores de SO y runtime local, con lease/fencing,
cancelación y fin de captura al perder propietario. CPU es el camino común;
aceleración es una capacidad comprobada del artefacto, no requisito NVIDIA.

Candidato inicial: Nemotron 3.5 ASR Streaming 0.6B y Nemotron 3 Diarization sobre
NeMo-Speech.cpp. Modelos preparados mediante manifiesto versionado/hash; sin
descargas implícitas del runtime al abrir. La [selección](../meeting-notes-models.md)
es revisable según evidencia en hardware e idioma objetivo. LLM de conclusiones
independiente y remoto sólo por selección explícita para este uso.

Sesión/nota canónica estables, bloque Meeting en documento v4 y migración aditiva
propuesta 021, a confirmar al aplicar. No reescribir documentos anteriores ni
convertir por título notas importadas. Original ASR, correcciones y conclusiones
son capas separadas; fuentes con revisión/offsets, no porcentajes de certeza inventados.

Detector local produce candidatos y popup, nunca una grabación por inferencia.
La UI ofrece Iniciar y omite el paso adicional por participante solicitado por
el usuario; los permisos del SO y el indicador visible se conservan. Abrir un
proceso no prueba una llamada. Backups requieren barrera de publicación y
restaurar una sesión interrumpida no vuelve a capturar.

## Consecuencias y alternativas

- Las notas funcionan sin modelos; la transcripción/conclusiones tienen estados
  propios y no se aparentan implementadas mientras falta su motor.
- Audio durable y versiones permiten corregir/reprocesar sin destruir el original;
  requieren espacio, cuotas y recuperación diseñadas antes de DDL.
- Un modelo generativo único mezcla errores; una solución sólo MLX no cubre Windows.
  ONNX/Qwen y Whisper quedan como alternativas comparativas.
- Identidad de voz no equivale a nombre. Etiquetas y vínculos se revisan, sin
  introducir huellas de voz o crear personas a partir de una suposición.
- Cada SO y calidad/latencia deben aceptarse por separado. La planificación y
  validación OpenSpec no acreditan captura, compatibilidad o calidad implementadas.

## Enlaces

[Diseño de producto](../meeting-notes-design.md),
[diseño técnico](../../openspec/changes/add-local-meeting-notes/design.md) y
[tareas propuestas](../../openspec/changes/add-local-meeting-notes/tasks.md).
