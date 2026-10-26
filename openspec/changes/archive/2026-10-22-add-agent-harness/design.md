# Design

## Contexto

Fase 5c conecta un proveedor de chat local con operaciones ya verificadas. No crea un segundo conjunto de reglas: cada herramienta se despacha por el núcleo y conserva sus validaciones. El harness es deliberadamente pequeño: un bucle síncrono de pasos acotados, sin framework externo ni ejecución de código.

## Ownership

- **Dominio**: entidades, documentos, lotes y retrieval siguen siendo dueños de sus reglas. El harness sólo invoca operaciones públicas.
- **Proveedor**: configuración efímera por solicitud (endpoint loopback, modelo, timeout, tokens, temperatura). No se persisten credenciales ni se activa nada al abrir el workspace.
- **Traza**: tabla operativa `agent_runs` (una fila por ejecución) con `trace_json` acotado. No es fuente de verdad ni historial de dominio; se puede borrar sin afectar entidades.
- **Reserva**: `lease_until` implementa concurrencia 1 por workspace. Una reserva vencida se considera abandonada y no bloquea indefinidamente.

## Protocolo

El proveedor recibe mensajes `system`/`user`/`assistant` en formato OpenAI-compatible (`POST /v1/chat/completions`). El system prompt enumera las herramientas permitidas con sus esquemas y exige JSON estricto, sin markdown:

```json
{"type":"tool_call","tool":"retrieval.search","input":{"query":"..."}}
{"type":"final","answer":"...","citations":[{"job_id":"...","generation_id":"...","ordinal":1,"quote":"..."}],"insufficient_evidence":false}
```

El harness parsea la respuesta, rechaza campos desconocidos y valida el nombre contra la lista. Un solo tool call por paso. El resultado se agrega como mensaje `user` con `{"tool_result": ...}` y el bucle continúa. Los modelos sin tool calling nativo pueden usar este protocolo JSON; no se depende de `tools` del proveedor.

## Herramientas

| Herramienta | Acceso | Condición |
| --- | --- | --- |
| `retrieval.search` | lectura | siempre |
| `entity.get` | lectura | siempre |
| `database.query` | lectura | siempre |
| `changes.preview` | lectura/check | siempre |
| `changes.apply` | escritura | `allow_apply=true` y recibo de un preview de la misma ejecución |

No se permite SQL, shell, workspace, adjuntos, exportación, extracción ni una operación no listada. El despacho valida el input con el DTO original (`serde_json::from_value` con `deny_unknown_fields`) y llama al método del workspace o a `batches::preview/apply`.

Para apply se guarda el `BatchPreview` de la ejecución y se exige que `plan_hash`/`base_hash` del recibo coincidan con el plan y el estado actual. `batches::apply` vuelve a validar todo y mantiene la atomicidad de fase 4.

## Citas

Durante la ejecución se acumulan los hits de `retrieval.search` por `(job_id, generation_id, ordinal)`. Al recibir `final`:

- Cada cita debe existir en ese conjunto.
- `quote` (1–1024 bytes) debe ser substring del texto del hit tras colapsar whitespace; no se aceptan citas aproximadas.
- Si `citations` está vacío, `insufficient_evidence` debe ser `true`; en ese caso la respuesta se acepta como limitación explícita.
- Si hay citas y `insufficient_evidence=true`, se rechaza por contradictorio.

La respuesta devuelve `citations` verificadas con localizador y fuente. No se afirma que el texto generado sea correcto: sólo que sus citas existen y coinciden.

## Límites

- Pasos: 1–12 (default 6).
- Contexto: 4 KiB–256 KiB (default 64 KiB); se cuentan system + pregunta + resultados.
- Resultado de herramienta: 4 KiB–256 KiB (default 64 KiB); si excede, se rechaza el paso.
- Timeout total: 1–120 s (default 60 s); timeout por request: 1–60 s.
- Tokens de salida: 1–4096 (default 1024).
- Traza: 1 MiB; se guardan resúmenes de request/response, no el contexto completo.
- Concurrencia: una ejecución con reserva vigente por workspace.

La cancelación es cooperativa: `agent.cancel` escribe `cancel_requested=1`; el bucle consulta antes de cada llamada y cada herramienta. El deadline se comprueba en los mismos puntos. Una ejecución cancelada no aplica cambios pendientes.

## Persistencia (migración 018)

```text
agent_runs(
  id TEXT PK, version, state, provider, model, endpoint,
  question, started_at, finished_at, lease_until,
  cancel_requested, step_count, answer, citations_json,
  trace_json, error
)
```

`state` ∈ `running|succeeded|failed|cancelled|limited`. `trace_json` contiene pasos con tipo, herramienta, duración, bytes, estado, error y resumen. Se indexa `(started_at,id)` para listar. No hay FK al dominio. El backup SQLite incluye la tabla; restore valida integridad y no reactiva reservas vencidas.

## Alternativas descartadas

- Framework de agentes externo: dependencia y superficie extra; el bucle acotado se prueba localmente.
- Tool calling nativo del proveedor: no todos los modelos locales lo soportan; el protocolo JSON es portable y validable.
- Diálogo sin herramientas: no permite citas verificables ni operaciones.
- Guardar el prompt completo: riesgo de datos y tamaño; se guardan resúmenes acotados.
- Permisos por prompt: no son aplicables; la política vive en Rust y el scope.

## Riesgos

- La calidad de la respuesta depende del modelo; las pruebas con servidor simulado sólo acreditan contrato, límites, citas y ausencia de writes indebidos.
- Un modelo puede producir JSON válido pero semánticamente pobre; el harness no lo presenta como verdad, sólo verifica citas.
- El proveedor local es un proceso del usuario; el endpoint debe ser confiable. Sólo se admite loopback y no se envían credenciales.
- El coste de tokens/tiempo no se mide contra un modelo real en este cambio.

## Verificación

- Unidad: validación de configuración, parser de protocolo, política de herramientas, verificación de citas, límites y cancelación.
- Integración: servidor de chat loopback simulado con respuestas guionadas (tool call y final), apply con recibo, traza, cancelación y concurrencia.
- Migración 018/backup/restore y preservación de datos.
- CLI/MCP: catálogo, scopes y errores.
- `fmt`, Clippy, contratos y OpenSpec estricto.
