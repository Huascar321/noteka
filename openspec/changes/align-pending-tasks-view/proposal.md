# Proposal

## Why

La vista All de Tasks en Home y en la página Tasks de Notion filtra los estados distintos de Done; Done reúne los completados. En Noteka, All incluye también tareas completadas, aunque existe una pestaña Done separada.

## What Changes

- Ofrecer una proyección compartida de tareas pendientes y usarla para la vista All de escritorio.
- Conservar el significado de `tasks_all` para los clientes existentes y el acceso de búsqueda a todas las tareas.
- Verificar estados, paginación, grupos y cambios de estado con datos sintéticos.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: All y Done distinguen tareas pendientes y completadas.
- `productivity-workflows`: proyección aditiva compartida de tareas pendientes, sin cambiar consultas existentes.

## Impact

Rust, esquema generado, interfaz y pruebas. Sin migración, cambios de datos, proveedor remoto ni cambios de consultas existentes. Las otras diferencias deliberadas y capacidades pendientes se revisan por separado. No sincronizar specs ni archivar.
