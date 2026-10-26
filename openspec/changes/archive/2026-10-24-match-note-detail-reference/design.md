# Design

## Context

La referencia autorizada se abrió con Playwright: icono grande, título, Tag/URL/Favorite/Type y relaciones antes de un bloque de código. La Note local equivalente conserva su ID, metadata local y prefijo reconocido del importador. El código actual duplica snapshots arriba y campos locales en otro sitio. Ver proposal.md y los deltas.

## Goals / Non-Goals

La entrega cambia presentación y acceso al editor de Notes normales; excluye páginas tipadas Goal/Milestone. No incorpora URL editable, Content, comentarios ni ejecución de código. Meeting Notes sigue 0/19 y no se captura audio.

## Decisions

1. **Propiedades locales frente al snapshot.** Un componente NotePageProperties recibe payload, relaciones reales y URL del prefijo. Favorite/Type usan el mismo draft y entity.update existente. Tag navega al original y abre el editor de relaciones ya existente para cambios; Project/People quedan debajo. Las propiedades importadas completas permanecen en View details con su carácter de origen. Alternativa descartada: editar el snapshot para simular metadata viva, porque contradice conservación/procedencia.
2. **Editor compartido y opciones explícitas.** BlockEditor recibe un contexto opcional para etiquetas de estructuras de Note, con Task como default previo. Un cuerpo de cero bloques se normaliza sólo en la vista y un solo párrafo vacío admite estructuras; no se persiste nada al abrir/enfocar. Enter usa focusStart y respeta composición IME. El callback existente recompone el prefijo intacto. Para el párrafo repetido observado se amplía la proyección sólo con fuente notion/Notes acreditada, marcador previo exacto, primer párrafo plano, tres anclas exactas, nombres conocidos únicos y sin contradicciones; cualquier ambigüedad sigue visible. Alternativa descartada: editor paralelo o conversión de documentos, porque duplicaría código y pondría IDs/formatos en riesgo.
3. **Estilos acotados.** Clase note-detail/note-document para propiedades, separación del cuerpo, código y checkboxes con data-checked. No modificar CSS previo ni defaults de entidades tipadas o Tasks. No añadir resaltador SQL ni dependencias sólo para copiar colores.
4. **Estado y operaciones.** Continúan CAS, guardias de navegación, conflictos, undo/redo y guardado explícito. Rust mantiene todas las reglas y validaciones existentes; no hay nuevas operaciones ni persistencia.

## Risks / Trade-offs

- Snapshot desactualizado → visible sólo como fuente en detalles; tipo/favorito y relaciones principales son locales.
- Ocultar un cuerpo importado ambiguo → el adaptador conserva su reconocimiento estricto y el prefijo se recompone sin modificarlo.
- Reemplazar estructura vacía existente → estructuras limitadas a un párrafo vacío, nunca headings/listas; copiar defaults de Task.
- Confundir revisión visual con persistencia → puente de lectura en copia restaurada, borradores descartados; compilar y registrar que no se ejercitó guardar ni IPC nativo. Sin suites nuevas/ejecutadas.

## Migration Plan

Sin migración ni cambio de formato. Baseline privado de fuentes y backup canónico antes del cambio. Revertir interfaz conserva SQLite/documentos; no restaurar un backup sobre ediciones posteriores. Capturas/evidencia personales quedan ignoradas y en copia privada.
