# ADR 0030: Harness de agentes local y acotado

- Fecha: 21 de octubre de 2026
- Estado: adoptado para fase 5c
- Ámbito: proveedor de chat, protocolo de herramientas, citas, límites, cancelación y trazas

## Contexto

Fase 4 ofrece `changes.preview/apply` y fase 5b recuperación con fuentes. Falta un bucle que conecte un modelo local con esas operaciones sin darle SQL, shell o acceso irrestricto al workspace, y que permita auditar qué hizo. No hay autorización para activar modelos o proveedores remotos; las pruebas deben usar servidores simulados.

## Decisión

1. **Proveedor local OpenAI-compatible** (`/v1/chat/completions`), sólo loopback, sin credenciales, con timeout, tokens y temperatura acotados. La configuración no se persiste en el dominio; cambiarla no requiere migración.
2. **Protocolo JSON estricto** con dos formas: `tool_call` (una herramienta) o `final` (respuesta + citas). No se depende de tool calling nativo; se validan campos desconocidos, tipos y tamaño.
3. **Herramientas públicas**: `retrieval.search`, `entity.get`, `database.query`, `changes.preview` y `changes.apply` sólo con `allow_apply` y un recibo de preview de la misma ejecución. No hay SQL, shell, archivos, workspace, importación, exportación ni extracción.
4. **Citas verificables**: cada cita debe referenciar un hit de retrieval de la ejecución y un `quote` que sea substring del fragmento (whitespace normalizado). Sin citas se exige `insufficient_evidence=true`.
5. **Límites**: 12 pasos, 64 KiB de contexto/herramienta por defecto (máximos 256 KiB), deadline 1–120 s, 4096 tokens y una ejecución concurrente por workspace mediante reserva.
6. **Persistencia**: migración aditiva 018 con `agent_runs`; traza JSON ≤1 MiB, retención 100 ejecuciones, sin contexto completo ni credenciales. `agent.trace` lee por ID o lista.
7. **Cancelación cooperativa** entre procesos (`agent.cancel` escribe una bandera; el bucle la consulta antes de cada llamada y herramienta).
8. **Errores**: validación de entrada → `validation_error`; proveedor/protocolo/citas → ejecución `failed` con traza; deadline/pasos/contexto → `limited`; cancelación → `cancelled`.

## Alternativas descartadas

- Framework de agentes externo: dependencia, superficie y comportamiento no verificados.
- Tool calling nativo: no portable a modelos locales sin esa capacidad.
- Diálogo sin herramientas: no permite citas verificables ni operaciones.
- Guardar prompts completos: riesgo de datos y tamaño; se guardan resúmenes acotados.
- Permisos por prompt: no son aplicables; la política vive en Rust y en el scope.

## Consecuencias

- La calidad de la respuesta depende del modelo; el harness sólo verifica contrato, política, citas y ausencia de writes indebidos.
- El servidor simulado de pruebas no acredita inferencia real ni calidad de citas.
- Un plan de cambio grande puede exceder el límite de herramienta; se rechaza, no se trunca.
- La reserva evita concurrencia pero no es un lock distribuido; un proceso muerto libera al vencer el plazo.

## Evidencia

`docs/verification-agent-harness.md` conserva pruebas, límites y pendientes. La activación real de un modelo local queda fuera de este cambio y requiere autorización específica.
