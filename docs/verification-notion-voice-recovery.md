# Verificación: recuperación del tipo Voice Note

El export puede nombrar el tipo `Voice Note`, mientras el núcleo usa `voice`.
El recuperador ahora reconoce esa etiqueta sin cambiar un tipo local existente;
las categorías desconocidas y las filas ambiguas no reciben un tipo inventado.

`python3 tests/recover-notion-voice.py` pasó contra CLI Rust y SQLite en un
directorio temporal: dry-run sin escrituras, rechazo de aplicar sin backup,
backup y revisión al aplicar, preservación de documentos e IDs, tipo existente,
tipo desconocido, identidad ambigua, consulta Voice y repetición idempotente.
No se ejecutó el recuperador sobre el workspace real ni se garantiza que un
export histórico tenga la fila inequívoca necesaria para cada nota.

La corrección no modifica datos existentes hasta una recuperación explícita
con las fuentes originales, revisión del informe y backup nuevo. La spec delta
permanece sin sincronizar y el cambio sin archivar.
