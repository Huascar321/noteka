## Context

WorkspacePane controla la identidad, guardias y herramientas; PageTools implementa el menú de Note y sus paneles. El resto de detalles usa la misma lista de grupos en el pie y archiva desde cabecera.

## Goals / Non-Goals

Unificar ubicación y comportamiento sin prometer funciones no disponibles ni duplicar herramientas. No extender el modelo de bloqueo de Note a otras entidades.

## Decisions

Reutilizar PageTools con placement header para toda entidad o borrador, aplicar los grupos existentes y sumar Archive/Restore a las entidades guardadas. Lock se agrega exclusivamente a Note normal. Quitar el grupo inferior y el botón de archivo previo para evitar duplicación. Preservar los paneles montados, guardias de borrador, capacidades por tipo, navegación accesible y restricciones de revisión del core.

## Risks / Trade-offs

El acceso a Archive cambia de lugar para otras entidades; se conserva el nombre y atajo existente. La verificación usa workspaces sintéticos, distintos modos de peek y anchos, además de compilación; no afirma aceptación nativa.
