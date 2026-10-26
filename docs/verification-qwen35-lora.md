# Verificación Qwen3.5-4B + LoRA (smoke)

Estado: **pipeline mínimo verificado localmente** el 22/10/2026. No es una
evaluación de calidad ni un benchmark de producto. El experimento sigue siendo
opcional y no condiciona el núcleo offline.

## Hardware y runtime

- Apple M1 Max, 64 GiB de RAM unificada, macOS.
- `mlx` 0.32.4 y `mlx-lm` 0.32.2 en un venv externo al repositorio.
- Modelo: `mlx-community/Qwen3.5-4B-4bit`, ~2.9 GB en disco.
- Los pesos y adaptadores viven en
  `$HOME/.local/share/noteka-verification/2026-10-21-phase5-201534/qwen-runtime/`;
  no se versionan.

## Entrenamiento

Fixture sintético determinista: 120 ejemplos de train, 30 de validación y 30 de
test. La tarea pide normalizar una etiqueta y devolver un JSON compacto con
`slug`, `color` y `priority`. La regla de color/prioridad es arbitraria y sirve
para comprobar que el adaptador aprende una convención, no para medir utilidad
real. Generación reproducible:

```sh
python3 scripts/prepare-qwen35-lora-fixture.py --output "$R/data-tags"
```

Configuración usada:

```yaml
model: <ruta>/Qwen3.5-4B-4bit
data: <ruta>/data-tags
fine_tune_type: lora
num_layers: 16
batch_size: 1
grad_accumulation_steps: 4
grad_checkpoint: true
iters: 150
max_seq_length: 512
learning_rate: 1e-5
optimizer: adamw
lora_parameters:
  rank: 8
  scale: 20.0
  dropout: 0.0
lr_schedule:
  name: cosine_decay
  warmup: 20
  arguments: [1e-5, 120, 1e-7]
```

Resultado medido:

- 150 iteraciones en **192 s**.
- Pico de memoria del proceso: **5,56 GB** (`/usr/bin/time -l`, RSS máximo ~3,9 GB).
- Adaptador final: **31 MB**, 8,1 M parámetros entrenables (0,193 % del total).
- La pérdida de validación bajó de ~3,0 a **0,215**.
- Carga con `mlx_lm.generate --adapter-path` y con `mlx_lm.server` verificada: el
  adaptador cambia las salidas del modelo base (de colores hex a nombres de la
  convención entrenada).

## Evaluación sintética

Script reproducible:

```sh
python3 scripts/evaluate-qwen35-lora.py \
  --model <ruta>/Qwen3.5-4B-4bit \
  --labels "$R/data-tags/labels.json" \
  --output "$R/eval-base.json"

python3 scripts/evaluate-qwen35-lora.py \
  --model <ruta>/Qwen3.5-4B-4bit \
  --adapter "$R/adapters-tags" \
  --labels "$R/data-tags/labels.json" \
  --output "$R/eval-adapter.json"
```

| Métrica en test (30) | Base | LoRA |
| --- | ---: | ---: |
| JSON exacto | 0,0 % | 3,3 % |
| `slug` | 96,7 % | 100 % |
| `color` | 0,0 % | 16,7 % |
| `priority` | 36,7 % | 40,0 % |

El adaptador se cargó y modificó el formato de salida, pero **no aprendió la
regla arbitraria de color/prioridad con 150 iteraciones**. No se debe presentar
esto como una mejora de calidad. La métrica `slug` ya era alta en el modelo base.

## Integración con el harness

`mlx_lm.server` expone chat-completions en loopback. El campo `model` de la
petición debe ser la **ruta local** del modelo; si se usa el nombre del repo,
el servidor intenta resolverlo en Hugging Face y falla con 401.

- `agent ask` con la pregunta trivial `Responde exactamente: hola` llegó al
  modelo, obtuvo un `final` JSON y terminó en `citation_required` al no incluir
  cita. Es el contrato esperado, no un fallo de transporte.
- Con una nota local que decía «La capital de Noteka es Aurora», tanto el modelo
  base como el adaptador respondieron `insufficient_evidence=true` sin llamar a
  `retrieval.search`. El transporte funciona; la calidad de uso de herramientas
  del 4B en este prompt es insuficiente.
- No se envió contenido a ningún proveedor externo; todo fue local.

## Límites y siguientes pasos

- No se evaluó Qwen3.5-9B, ni una comparación con modelos externos bajo las
  mismas herramientas.
- No se entrenó para tool-calling ni para tareas reales de Noteka.
- El adaptador de etiquetas no debe cargarse para QA general: es un experimento
  acotado.
- Rank 8 y 150 iteraciones son un smoke; el plan menciona rank 32 y una
  evaluación más amplia, todavía pendiente.
- No se integró un runtime MLX al producto ni se agregó una dependencia Python
  al núcleo Rust.
