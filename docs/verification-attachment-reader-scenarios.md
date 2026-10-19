# Matriz del lector de generaciones

Cuatro requisitos/ocho escenarios de `add-local-attachment-reader-ui`.
Las lecturas reales provienen de CLI/SQLite con golden Python independiente;
no son pruebas de IPC Tauri. [Resultados](verification-attachment-reader-ui.md).

| Escenario | Implementación | Evidencia |
| --- | --- | --- |
| Generación sin seleccionar para búsqueda | Tres operaciones de lectura por ID/tipo, sin consultar índice | Cliente comprueba payloads exactos; fixture real sin selecciones y snapshots SQL/archivos idénticos tras cada lectura. |
| Abrir editar y refrescar sin consulta | Constructor/edit sin llamadas; panel sin polling/activación | Cliente montaje/edición/dispose; headless compara registro de operaciones al abrir/editar y remonta con nota pendiente. |
| Recorrido completo y vacío | Página reemplazada de veinte, cursor ligado a generación | Cliente páginas/vacío; headless UTF-8 largo, 23 páginas PDF y 25 celdas XLSX, recorrido completo y generaciones vacías. |
| Fuente derivado o cursor inválidos | Revalidación Rust y descarte íntegro de página/cursor | Cliente errores/identidad; headless fuente alterada, derivado corrupto de igual longitud, regeneración entre páginas y ID/tipo inválidos. |
| Texto PDF y UTF-8 literal | Pre escapado y localizadores tipados; manifiesto completo | Golden compara Unicode/BOM/CRLF, HTML aparente, texto largo y páginas no_text, sin recortar ni atribuir offsets fuente al PDF derivado. |
| Celdas y metadatos XLSX | Tarjetas dispersas y manifiesto literal; caché/fórmula separados | Golden compara cadenas/números léxicos/estilos/fechas, fórmulas shared/array, visibilidad/merges y coordenadas extremas sin expandir/recalcular. |
| Respuesta vieja y remount | Lifetime y cupo físico compartido uno | Cliente éxito/error tardío, remount y espera cancelada; headless respuesta demorada al salir/cambiar nota no publica en contexto nuevo. |
| Temas teclado y buffer | Integración key workspace/nota/epoch; guardia App intacta | 870 checks de producto en ambos temas, 1280/900 px, foco/teclado, buffer pendiente y cancelar/fallo de guardado. Cero requests externos. |

Los 848 checks aislados anteriores se conservan como otro hito; no se suman a los
870 integrados. No se atribuye al lector render visual, OCR, recálculo o compatibilidad universal.
