# Proposal

## Why

Week y Month en Tasks de la referencia son calendarios por Due navegables y con acceso a tareas sin fecha. Las vistas locales son listas estáticas de tareas pendientes en el período actual.

## What Changes

- Ofrecer calendarios semanales y mensuales por Due en Home y Tasks, con navegación, tareas sin fecha y fechas incompatibles conservadas.
- Consultar el período seleccionado antes de paginar; mantener aviso de resultados parciales y operaciones de apertura y captura explícita.
- Conservar `tasks_week` y `tasks_month` para clientes existentes.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: Week y Month como calendarios navegables.
- `productivity-workflows`: consultas aditivas por período civil y sin fecha.

## Impact

Rust, contratos generados, interfaz y pruebas sintéticas; sin migración de datos, sync ni archivo. No se adoptan automatizaciones ni recurrencias remotas.
