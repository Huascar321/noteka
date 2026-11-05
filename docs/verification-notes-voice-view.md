# Verificación: Voice en Notes

La regla Notes de Notion inspeccionada el 05/11/2026 incluye Voice Note, incluso
cuando tiene URL. La interfaz local usa la nueva proyección `notes_general`, que
incluye Voice de la biblioteca; `notes_library` conserva su regla anterior y la
vista Voice abre las mismas entidades. Clips, Journal y Meetings no se incluyen
en Notes. No se implementó grabación ni transcripción.

La referencia también filtra por URL y distingue Daily. La URL importada local
puede vivir en bloques del documento y Home no los expande; este cambio no
supone que la URL esté vacía ni promete paridad de ese filtro. No se modificaron
notas ni se importaron contenidos de Notion.

- Rust Home: 13 pruebas aprobadas, con filtro antes de paginar, Voice, origen,
  archivo, Goal, cursor incompatible y consulta histórica.
- Chromium y WebKit: 8 checks por motor para Notes y Voice en Home y en la
  página Notes, misma entidad y consulta anterior, con datos sintéticos y sin
  solicitudes externas. No se verificó composición nativa Tauri/macOS.
- Contratos generados, build y OpenSpec estricto aprobados; Vite mantiene su
  aviso de chunk. Delta sin sync ni archivo.
