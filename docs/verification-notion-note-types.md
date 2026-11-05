# Verificación: tipos de Notes

El selector de la referencia distingue Daily, Journal, Book, Recipe, Outlier
Content, Creator Tips y Tools. El núcleo los admite sin migrar datos. Journal
de Home y Notes usa `notes_diary`, con Daily y Journal; `notes_journal` mantiene
su filtro anterior. Notes generales excluye ambos diarios. Las etiquetas de
selección y listas son compartidas. El recuperador reconoce estos tipos en una
fila inequívoca sin reemplazar una clasificación existente.

- Rust Home: 14 pruebas aprobadas; se verificó pertenencia, archivo, paginación,
  cursor incompatible, distinción de tipos y consulta anterior.
- Chromium y WebKit: 15 checks por motor en UI de producción con datos sintéticos,
  incluyendo ambas superficies, Daily, Journal, Book y opciones de selección.
- Recuperación con CLI/SQLite sintéticos: dry-run, backup, CAS, fuentes ambiguas,
  todos los nuevos tipos y preservación de documentos aprobados.
- Contratos regenerados y build aprobados; Vite mantiene el aviso de chunk grande.

El enum ampliado exige clientes actualizados para editar los nuevos tipos. No se
ejecutó recuperación en el workspace real. URL sigue sin modelarse íntegramente
en las proyecciones, y el cambio no prueba sincronización con Notion.
