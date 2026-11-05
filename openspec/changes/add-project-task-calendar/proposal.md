# Proposal

## Why

El detalle de Project inspeccionado en Notion ofrece Tasks en List, Board y Calendar mensual por Due. Noteka sólo ofrece List y Board pese a disponer de fechas civiles y relaciones locales.

## What Changes

- Agregar Calendar mensual de tareas vinculadas al Project, con navegación, apertura de originales y acceso a tareas sin fecha.
- Permitir preparar una tarea para un día conservando el vínculo de Project y el guardado explícito.
- Indicar cuando la proyección muestra sólo relaciones cargadas. No inferir proyectos padres ni descargar datos remotos.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: calendario local de Tasks en el detalle de Project.

## Impact

Interfaz y pruebas sintéticas con operaciones existentes; sin migración ni contrato nuevo. La referencia también incluye tareas de proyectos padres, relación que no está modelada localmente y no se inventa. Sin sync ni archivo.
