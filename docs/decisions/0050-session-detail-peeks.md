# ADR 0050: presentación de detalles sobre panes de sesión

Fecha: 26/10/2026. Estado: adoptada para implementación local.

## Context

El usuario pidió Side peek derecho, Center peek, Full page y New tab, con modo
predeterminado configurable. WorkspacePane ya concentra buffer, revisión,
editor y guardia; recrearlo al cambiar de modo perdería historial y selección.

## Decision

App mantiene una única colección con claves estables de panes. Un detalle puede
tener un dueño de sesión y una presentación Side/Center/Full. Sólo las raíces se
muestran en la barra. New tab promueve la misma instancia a raíz; cambiar a peek
desde una pestaña independiente crea un origen de sección sin recrear el editor.
Hay un hijo por raíz, sin peeks anidados. Reemplazar el destino reutiliza la
guardia del hijo; todos los panes participan en los cierres agregados.

Foco de atajos, visibilidad y activación de pestaña son estados diferentes.
Cambiar el foco entre origen y Side peek no vuelve a consultar/desmontar las
filas del origen; la actualización explícita o reactivación de pestaña conserva
las reglas de lectura vigentes. Un origen cubierto por Full page permanece
montado e inerte, con sus guardias nativas disponibles en el cierre agregado.

El modo predeterminado es una preferencia global local validada del cliente,
inicialmente Side peek. Las sesiones no se persisten. No cambia SQLite, contratos,
documentos, guardado explícito ni autoridad de Rust.

## Alternatives

- Árboles separados para Side/Center/Full: descartados porque desmontan el editor.
- Serializar el borrador y volver a abrirlo por modo: no conserva selección,
  composición ni historial, y aumenta las oportunidades de pérdida de datos.
- Preferencias por vista: fuera del alcance explícito actual; la UI identifica
  el ámbito global para evitar prometer esa granularidad.

## Consequences

La presentación comparte buffers y cierre protegido; el shell debe recorrer
también hijos ocultos de pestañas inactivas y separar teclado de visibilidad.
El ancho puede redistribuir texto, pero el cambio de modo no genera revisiones.
Los cambios guardados se reflejan en las proyecciones del origen al actualizar
o reactivar su pestaña, sin polling ni refresco destructivo por mover el foco.
