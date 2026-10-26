# Meeting Notes: selección inicial de modelos

Investigación al **24 de octubre de 2026**. Es una propuesta; no se instalaron
motores/modelos ni se midió su calidad en Noteka.

## Recomendación

Empezar con **Nemotron 3.5 ASR Streaming 0.6B** para texto y **Nemotron 3
Diarization** para separar voces, mediante **NeMo-Speech.cpp**. La combinación
ofrece un runtime C++ común con CPU y aceleración según plataforma. El resumen
usa un perfil LLM independiente: un ASR no reemplaza ese trabajo.
[Runtime oficial](https://github.com/NVIDIA/NeMo-Speech.cpp).

| Candidato | Qué aporta | Decisión propuesta |
|---|---|---|
| [Nemotron 3.5 ASR Streaming 0.6B](https://huggingface.co/nvidia/nemotron-3.5-asr-streaming-0.6b) | Streaming, español incluido entre sus locales listos; publicado en junio de 2026. | Primera opción ASR por integración nativa común. La tarjeta distingue 19 locales listos de otros que requieren adaptación; no tratar los 35 etiquetados como equivalentes. |
| [Nemotron 3 Diarization](https://huggingface.co/nvidia/Nemotron-3-Diarization) | Publicado el 23/09/2026; diarización online/offline, hasta ocho hablantes. | Primera opción para actores, con etiquetas revisables y límite visible. No aporta nombres reales. |
| [Qwen3-ASR 0.6B/1.7B](https://github.com/QwenLM/Qwen3-ASR) | Familia publicada en enero de 2026 y soporte Transformers nativo en junio; español y streaming. | Alternativa comparativa de ASR. 1.7B debe justificar coste en hardware objetivo; no asumir que el tamaño garantiza mejor Meet. |
| [Qwen3-ASR con sherpa-onnx](https://k2-fsa.github.io/sherpa/onnx/qwen3-asr/index.html) | Exportación ONNX y plataformas Mac/Windows, incluso CPU. | Vía alternativa común si favorece calidad/rendimiento real. No incorporar dos runtimes por defecto sin esa evidencia. |
| [pyannote Community-1](https://huggingface.co/pyannote/speaker-diarization-community-1) | Diarización offline y alineación exclusiva para reconciliar texto. Acceso sujeto a aceptar condiciones de Hugging Face. | Comparador para diarización; no es la dependencia inicial por instalación/runtime y acceso adicional. |
| [Whisper.cpp](https://github.com/ggml-org/whisper.cpp) | Ecosistema de ASR local con amplia portabilidad. | Referencia de comparación madura; separar transcripción y diarización. No elegir sólo por antigüedad/popularidad. |

## Portabilidad y preparación

La [instalación de NeMo-Speech.cpp](https://github.com/NVIDIA/NeMo-Speech.cpp/blob/main/docs/install.md)
documenta artefactos para Mac/Windows, CPU y backends de GPU. Sus releases pueden
ir detrás de main: la implementación debe fijar una versión y comprobar las
capacidades requeridas en cada SO, sin seguir nightly automáticamente.

Su [CLI](https://github.com/NVIDIA/NeMo-Speech.cpp/blob/main/docs/cli.md) descarga
modelos predeterminados si faltan. Noteka debe preparar modelos antes del inicio
y pasar rutas locales explícitas; no ejecutar ese comportamiento por abrir una
nota. `transcribe --live` obtiene micrófono: para una llamada hace falta integrar
buffers nativos de audio remoto con el SDK de streaming y mantener sus relojes.

La UI muestra tamaño/licencia/huella reales del manifiesto preparado. Parámetros
del modelo, tamaño de descarga, RAM y velocidad son magnitudes distintas.
No se instala CUDA para exigir funcionamiento básico en un equipo sin NVIDIA.

## Recepción de la comunidad y límites de la evidencia

Snapshot público de las tarjetas/API de Hugging Face: ASR candidato, **1.231 likes**;
diarizador, **805**; Qwen3-ASR-1.7B, **1.207**. Son señales de adopción, no mediciones
de precisión o satisfacción ni consenso sobre reuniones en español.

Se priorizaron experiencias directas y comparativas con resultados disponibles.
La [comparativa de 15 conversaciones](https://www.reddit.com/r/LocalLLaMA/comments/1wq1as8/i_compared_diarization_models_on_15_clinical/)
incluye audios/referencias/salidas y comentarios con resultados diferentes; su
runtime propio no está publicado. El [notetaker local para Mac](https://www.reddit.com/r/LocalLLaMA/comments/1wu2lpa/five_local_models_68_gb_of_weights_my_opensource/)
describe otra combinación Qwen/Nemotron y hardware concreto. La recepción disponible
justifica candidatos, no una garantía trasladable a otra Mac o a Windows. Los
hilos pequeños no se presentan como un consenso amplio.

## Decisión antes de adoptar definitivamente

Revisar con audio acreditado en español habitual: nombres propios, reuniones de
varias voces, acentos, solapamiento, micrófono y audio remoto, frases cortas y
silencio. Comparar texto, atribución, latencia observada, coste en CPU/RAM y
recuperación. Mantener un candidato común entre sistemas y documentar diferencias.
Los benchmarks del proveedor o de una GPU ajena no acreditan esas condiciones.

Conclusiones y responsables propuestos deben revisarse contra tramos reales,
incluyendo ausencias de acuerdo y cambios de hablante. No prometer exactitud
perfecta ni hacer depender el producto del experimento LoRA ya existente.
