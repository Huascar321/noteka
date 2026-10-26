# Proposal

## Why

Dos de tres gates mixtos superaron300ms p95 con picos aislados; los diagnósticos
no demuestran una regresión del lector de tablas. Sí muestran trabajo redundante:
la verificación materializa/tokeniza8MiB completos antes de tokenizar las clases
de textos idénticos para el camino exacto de postings ya existente. Se necesita
margen sin reducir ninguna comprobación o cuota.

## What Changes

- Comparar todas las filas/campos/tipos de la copia FTS contra los fragmentos
  canónicos ya validados, ordenados por rowid, sin construir primero otro FTS.
- Reutilizar la comprobación exacta de postings por clases idénticas. Conservar
  sus clases canónicas sólo durante el snapshot para comprobar ausencia global;
  materializar el FTS esperado completo para fallback o contraste independiente
  de candidatos/puntajes. No inferir ausencia a partir del índice real solamente.
- Contrastar tamaños de tokenización por fila y totales FTS5 con el canónico,
  también sin hits. Las pruebas detectaron que el verificador previo delegaba
  algunas alteraciones de estadísticas al contraste de candidatos.
- Contrastar contra el algoritmo anterior independiente en corpus válidos y
  corrupciones, medir recursos y repetir dos pares de gates originales intactos.

## Non-goals

No caches entre llamadas, hashes de confianza, muestreo, cambios de tokenización,
SQLite/API/MCP/UI, migraciones, datos persistentes, cuotas/targets/deadlines,
helpers, modelos o Home. No declara terminado el lector de tablas6/8 ni habilita5b.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

Ninguna: refactor de implementación de los requisitos existentes de integridad
completa/recursos de `local-text-search`. `skip_specs: true`; no se inventa un
requisito ni se sincronizan deltas de otro cambio incompleto.

## Impact

Sólo verificador Rust, tests diferenciales y documentación de rendimiento. Baseline
diagnóstico16/API57 preservado antes del cambio (671 fuentes/14 binarios); API57
incluye el lector en aceptación, no es un cierre previo. Se conserva también el
cliente56 independiente. El usuario autorizó completar autónomamente el ciclo5a;
la optimización es necesaria para sus gates fijos300/150, sin límite horario.
