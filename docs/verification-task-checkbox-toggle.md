# Verificación del checkbox de tareas

31/10/2026. Las marcas de estado de Tasks eran `span` decorativos
(`aria-hidden`) dentro del botón que abre la tarea: al hacer clic se abría el
detalle y el estado no cambiaba. En la vista All, las tareas completadas usaban
verde y las pendientes el acento azul, pero ambas seguían sin ser controles.

## Cambio

- La marca se separó en un botón `role="checkbox"` con `aria-checked` y
  etiqueta “Mark complete/pending: título”. El checkbox es hermano del botón de
  apertura dentro de `.home-task-main`; no está anidado.
- `HomeTaskSection` actualiza `state` con `entity.update` y un parche
  `{state: 'todo'|'done'}`; Rust fusiona el payload y valida. Después refresca
  Tasks y Projects mediante `onchanged` (`refreshPanel` en TasksPage/HomePanel).
- El control se deshabilita mientras hay una operación en curso o la lectura está
  ocupada. Un fallo se muestra con `role="alert"`.
- El pendiente es un cuadrado vacío con borde neutro y el completado un check
  verde; la diferencia no depende solo del color. El detalle de proyecto
  (`ProjectContents`) todavía usa su fila de lectura y queda como follow-up.

## Evidencia

En Chromium y WebKit headless se crearon dos tareas. En la vista **All**
agrupada, el checkbox pendiente tenía `role="checkbox"`, `aria-checked="false"`,
no estaba dentro de `.home-open` y su etiqueta incluía el título. Al clickearlo:

- pasó a `aria-checked="true"` y “Mark pending”;
- la marca pasó de un cuadrado vacío (`rgba(0, 0, 0, 0)`, borde neutro) al
  verde `rgb(125, 159, 145)`;
- `entity.get` devolvió `state: "done"` y revisión 2.

En **Inbox**, una tarea pendiente plana se completó, desapareció de la vista y
`entity.get` devolvió `state: "done"` y revisión 2. No se inspeccionó la ventana
nativa ni se midió CPU/GPU.

El follow-up de `ProjectContents` quedó resuelto en `add-project-task-board`:
List usa `ProjectTaskRow` con checkbox real y el Board usa el estado de la tarea.
Ver [verificación del Board](verification-project-task-board.md).
