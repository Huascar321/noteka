# Tasks

## 1. Línea de base

- [x] 1.1 Preparar y ejecutar benchmark de Home con 1.000 y 8.000 entidades sintéticas, 30 ciclos y memoria; conservar resultados anteriores a la actualización.
- [x] 1.2 Registrar los fallos existentes de Home y su relación con el baseline anterior, sin modificar aserciones para ocultarlos.

## 2. Motor incluido

- [x] 2.1 Actualizar rusqlite y el lock a SQLite 3.54.1; compilar y verificar la versión efectiva de desktop, CLI y MCP.
- [x] 2.2 Verificar integridad, preservación de formatos desconocidos, backup/restauración, migraciones y concurrencia con fixtures sintéticas.
- [x] 2.3 Repetir el benchmark y documentar medidas, límites, compatibilidad y procesos antiguos; validar este cambio OpenSpec.
