# Presentación de detalles

Actualización: 26/10/2026. Notes, Tasks, Projects, Tags, People, Goals y Milestones
comparten el selector **Switch peek mode** en la cabecera.

- **Side peek** abre el detalle a la derecha y conserva la vista de origen a la
  izquierda. Su borde izquierdo permite ajustar el ancho, con máximo 65% de la
  ventana; al superar 35% se alterna con el menú izquierdo.
- **Center peek** muestra un marco centrado con fondo atenuado y foco contenido.
- **Full page** usa el área de trabajo completa y conserva el origen en sesión.
- **New tab** convierte ese detalle en una pestaña independiente.

La misma instancia del editor permanece montada entre modos, incluida la
promoción a pestaña: buffer, revisión capturada, selección e historial no se
reconstruyen ni se guardan automáticamente. El cambio de ancho puede redistribuir
líneas; el origen conserva su contexto y anclaje de scroll.

**Default opening mode** permite elegir el modo para nuevas aperturas. La etiqueta
**All views · New openings** indica su ámbito global para la aplicación, sin
preferencias separadas por vista. Se guarda en `noteka.peek-mode.v1` del cliente;
un valor desconocido o acceso fallido usa Side peek. Un fallo de escritura muestra
un aviso y conserva la elección durante la sesión. Cambiar el predeterminado no
altera el detalle actual ni documentos del workspace.

Cerrar un peek, reemplazar su contenido o abandonar su pestaña/workspace/ventana
usa Save and continue, Discard o Cancel. Un error o conflicto mantiene el buffer.
Los peeks de pestañas inactivas también participan en el cierre agregado. Escape
cierra primero menús y luego solicita el cierre protegido del peek. New tab no
necesita descartar ni guardar, porque conserva el mismo editor.

Los peeks y sus pestañas son estado de sesión; no se reabren tras reiniciar la
aplicación. Los documentos siguen en SQLite y sus operaciones/validaciones en Rust.

Los [anchos de panel](panel-widths.md) se recuerdan localmente. Abrir el menú con
un Side peek ancho lo oculta conservando el editor; no equivale a cerrar o
descartar. La expansión/contracción usa la misma cadencia de 260ms del menú y
respeta movimiento reducido. Ver [ADR0051](decisions/0051-panel-widths-and-suppressed-peeks.md).

La verificación se documenta en [verification-detail-peek.md](verification-detail-peek.md).
