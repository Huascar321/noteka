# Proposal

## Why

Deadlines en Projects de Notion es un calendario mensual por Target Deadline que incluye proyectos con cualquier estado y ofrece acceso a los que no tienen fecha. La vista local muestra sólo una lista de proyectos incompletos con fecha.

## What Changes

- Mostrar Deadlines como calendario mensual con proyectos activos locales y de origen en todos los estados, acceso a los sin fecha y cobertura parcial explícita.
- Conservar la consulta pública `projects_deadlines` y añadir una proyección para la interfaz, sin cambiar datos o contratos anteriores.
- Permitir preparar un proyecto para un día con guardado explícito y verificar fechas, estados, cursor y controles.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: calendario de Deadlines en Projects.
- `productivity-workflows`: proyección aditiva de proyectos para el calendario.

## Impact

Rust, contratos generados, interfaz y pruebas. Sin migración ni modificación de datos. Sin sync ni archivo.
