# Corpus XLSX de búsqueda (MIT)

Datos sintéticos propios, sin archivos personales ni descargas. Los originales se
generan fuera de Git. `generate.py` reutiliza los **expected** independientes de
los casos admitidos del lector, sin modificar sus 216 casos ni sus pins, y agrega
512/513 campos, Unicode largo, caché/fórmula separadas y contenido hostil inerte.
Su oracle Python no invoca el proyector Rust. `golden.sha256` fija generador y
manifest; `scripts/test-xlsx-search.py` contrasta hash/filas/hits/localizadores con
la CLI real, retirando el helper después de extraer.

`benchmark.json` describe un corpus separado: 2 MiB UTF-8 + 2 MiB PDF + 4 MiB XLSX,
con campos cortos/largos, Unicode, fórmulas, hojas ocultas y >256 términos. Sus
pins de originales/proyección se fijan antes de medir; no se alteran para mejorar
resultados. El gate anterior no cambia. Las fuentes, workspaces, logs, RSS y
mediciones permanecen en temporales externos.
