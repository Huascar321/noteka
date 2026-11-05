# ADR0054 — Bloqueo de edición persistente y apertura temporal

Fecha: 26/10/2026. Estado: adoptada para `add-note-page-lock`.

`page_locked` es un booleano opcional de NoteFields en el payload original de
SQLite. Rust valida su tipo y entity.update conserva CAS/auditoría. No se crea
tabla, operación ni migración SQL: Notes previas sin el campo no se reescriben.
Ausencia/null equivale a no bloqueada; false se admite como desactivación explícita.
La interfaz usa un parche de payload que omite título y documento al cambiarlo.

La preferencia evita ediciones accidentales de la Note en el escritorio. No es
una autorización ni impide operaciones deliberadas por CLI/MCP. Locked permite
edición temporal sólo en la instancia actual; ese permiso efímero no se guarda
ni exporta. Re-lock resuelve primero el borrador pendiente y restaura restricciones.
Guardar cambios temporales conserva el booleano true para la próxima reapertura.

Se descartan localStorage como autoridad, modificar documentos para marcar modo
de edición y guardar un desbloqueo temporal como false: separarían el estado del
original o perderían el propósito de re-bloquear al reabrir. Se mantienen los
lectores y la selección/copia; las operaciones que cambian la Note requieren
desbloquear. No se agrega un ACL al núcleo con una excepción oculta para la UI.

Compatibilidad: get/export/backup manejan el payload original y conservan el
campo. Binarios anteriores estrictos pueden rechazar parches de payload que lo
incluyan; no deben borrar campos para hacerlos pasar. Las actualizaciones que
omiten payload conservan su JSON original, incluido contenido futuro. La reversión
de código mantiene el dato recuperable y no ejecuta un barrido destructivo.
Las pruebas cubren ausencia, tipo inválido, CAS, preservación y backup/restauración.
