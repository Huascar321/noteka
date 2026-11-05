# Verificación: calendario de Tasks en Project

El 05/11/2026 se inspeccionó en lectura la vista Calendar del Project en Notion:
usa Due, escala Month, siete días y navegación anterior/Today/siguiente; permite
ver elementos sin fecha. Su filtro remoto incluye además una relación Parent
Project que Noteka no modela. El calendario local usa solamente vínculos reales
del Project y no afirma paridad con esa jerarquía ni sus automatizaciones.

- El calendario usa fechas civiles, incluye tareas Done y separa las sin fecha
  y fechas no interpretables. El mes actual sigue la zona del workspace.
- Una acción por día prepara un borrador con Due y contexto Project. Abrir o
  descartar no escribe; Save utiliza las mismas operaciones y guardias existentes.
- Si existe cursor de relaciones se indica la cobertura parcial; Load more usa
  el cursor compartido. No se presenta la lista cargada como base completa.
- Dos pruebas puras cubren cambio de zona, bisiestos, límites de año y fechas
  inválidas. Chromium y WebKit aprobaron 19 checks cada uno para navegación,
  apertura, captura/Save/Discard, vínculo, carga parcial, foco desplazable,
  tema oscuro y 420 px. El Board anterior pasó cinco checks por motor.
- Build y validación estricta de OpenSpec aprobados; Vite conserva su aviso de
  tamaño de chunk. No se verificó IPC ni composición Tauri/macOS nativos.

Durante la automatización se corrigieron esperas de carga, un selector que no
contemplaba el texto Done y la ubicación del control Load more en el footer.
La revisión de una captura sintética detectó una etiqueta Done que envolvía
verticalmente en una celda estrecha; se corrigió y se repitieron los recorridos.
No se editaron datos de Notion. Specs sin sincronizar y cambio sin archivar.
