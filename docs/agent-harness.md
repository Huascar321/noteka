# Harness de agentes local

Fase 5c conecta un proveedor de chat local con operaciones públicas del núcleo. No agrega reglas de dominio: cada herramienta usa el DTO y la validación existentes. [ADR 0030](decisions/0030-agent-harness.md).

## Operaciones

### `agent.ask`

```json
{
  "question":"¿Qué pendientes tengo de CRE?",
  "provider":{
    "runtime":"openai_chat_completions_v1",
    "endpoint":"http://127.0.0.1:8080/v1/chat/completions",
    "model":"modelo-local",
    "timeout_ms":30000,
    "max_tokens":1024,
    "temperature":0.0
  },
  "limits":{"max_steps":6,"max_context_bytes":65536,"max_tool_bytes":65536,"deadline_ms":60000},
  "allow_apply":false
}
```

Devuelve `run` (estado, pregunta, proveedor/modelo, tiempos, error, respuesta), `steps` y `citations` verificadas. Estados: `running`, `succeeded`, `failed`, `cancelled`, `limited`. Una falla de proveedor/protocolo/citas devuelve `run.state=failed` con traza consultable; no presenta la respuesta como válida.

### `agent.cancel`

```json
{"id":"<uuid>"}
```

Marca una ejecución `running` para cancelación cooperativa. El bucle la observa antes de cada llamada y herramienta; no aplica cambios pendientes.

### `agent.trace`

```json
{"action":"get","id":"<uuid>"}
{"action":"list","limit":20,"cursor":null}
```

`get` devuelve la ejecución y sus pasos; `list` pagina ejecuciones recientes. La traza guarda resúmenes acotados, no el contexto completo ni credenciales.

## Protocolo del modelo

El system prompt exige JSON estricto:

```json
{"type":"tool_call","tool":"retrieval.search","input":{"query":"CRE","mode":"hybrid"}}
{"type":"final","answer":"...","citations":[{"job_id":"...","generation_id":"...","ordinal":1,"quote":"texto exacto del fragmento"}],"insufficient_evidence":false}
```

Una herramienta por turno. Se rechazan campos desconocidos, tipos incorrectos, JSON inválido y nombres fuera de la lista. El resultado de la herramienta se agrega como mensaje `user` con `{"tool":...,"result":...}`.

## Herramientas

| Herramienta | Permiso | Condición |
| --- | --- | --- |
| `retrieval.search` | lectura | siempre |
| `entity.get` | lectura | siempre |
| `database.query` | lectura | siempre |
| `changes.preview` | lectura/check | siempre |
| `changes.apply` | escritura | `allow_apply=true` y recibo de un preview de la misma ejecución |

No hay SQL, shell, archivos, workspace, importación, exportación ni extracción. `changes.apply` conserva la atomicidad de fase 4; el recibo debe coincidir con el plan y el estado.

## Citas

Una cita es válida si:

1. `(job_id, generation_id, ordinal)` corresponde a un hit de `retrieval.search` de la misma ejecución.
2. `quote` (1–1024 bytes) aparece en el texto del fragmento tras colapsar whitespace.

Sin citas se exige `insufficient_evidence=true`; con citas y `insufficient_evidence=true` se rechaza. El harness verifica existencia y coincidencia, no verdad semántica ni calidad del modelo.

## Límites

- Pasos 1–12; contexto 4–256 KiB (default 64 KiB); herramienta 4–256 KiB.
- Deadline 1–120 s; timeout de proveedor 1–60 s; tokens 1–4096; temperatura 0–2.
- Traza ≤1 MiB; retención 100 ejecuciones; una ejecución concurrente por workspace (reserva).
- Respuesta ≤64 KiB; cita ≤1 KiB.

## Endpoint

Sólo loopback `http://127.0.0.1:puerto/ruta` o `[::1]`. Sin TLS, credenciales, redirecciones ni red externa. El proveedor debe exponer `/v1/chat/completions` estilo OpenAI con `choices[0].message.content`. No se usa `tools` nativo: el protocolo JSON es portable a modelos locales sin tool calling.

## CLI

```sh
noteka --json --workspace "$W" agent ask --input ask.json
noteka --json --workspace "$W" agent trace --input '{"action":"list"}'
noteka --json --workspace "$W" agent cancel --input '{"id":"..."}'
```

## Verificación y pendientes

Ver [verificación](verification-agent-harness.md). Las pruebas usan un servidor loopback simulado con respuestas guionadas; acreditan contrato, políticas, citas, límites, cancelación y trazas, no inferencia real. Activar un modelo local y evaluar calidad requiere autorización y evidencia separadas.
