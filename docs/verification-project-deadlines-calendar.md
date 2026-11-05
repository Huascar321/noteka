# Verificación: calendario de Deadlines

La vista Deadlines de Notion inspeccionada el 05/11/2026 usa Month por Target
Deadline; sólo filtra archivo y mantiene un acceso a proyectos sin fecha. El
calendario local usa `end_date`, incluye todos los estados y separa sin fecha y
fechas inválidas. No reproduce automatizaciones ni importa datos remotos.

`projects_calendar` se añadió a `home.query`. Excluye archivo local y de origen,
ordena antes de paginar y conserva estados Done y ausencia de fecha. La consulta
anterior `projects_deadlines` sigue seleccionando sólo proyectos no Done con
fecha. Clientes antiguos mantienen sus consultas; requieren el esquema actualizado
para usar la nueva variante. Los días preparan proyectos Planned con ese plazo;
no se escribe hasta Save. Las páginas pendientes se anuncian y se cargan con el
cursor existente.

- La suite Rust `home`: 11 pruebas aprobadas; la nueva cubre Done, sin fecha,
  fecha inválida, archivo, cursores y compatibilidad de la consulta anterior.
- Chromium y WebKit: 20 checks por motor en Projects y Projects dentro de
  Tasks, navegación, apertura de categorías, borrador/Discard/Save, página
  parcial, tema oscuro, 420 px y foco desplazable; sin solicitudes externas.
- Build y validación estricta de OpenSpec aprobados, sin errores/avisos Svelte;
  Vite mantiene el aviso de chunk grande. Capturas y fixtures son sintéticas.

No se verificó composición ni IPC nativo de Tauri/macOS. El delta no se
sincronizó ni se archivó.
