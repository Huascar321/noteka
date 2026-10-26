# Revisión del detalle de Note

Fecha: 24/10/2026. Cambio `match-note-detail-reference`.

## Referencia y diagnóstico

Se abrió una Note existente desde la biblioteca Notes de Notion con Playwright,
se pasó a página completa y se capturaron su detalle y View details. No se
editó ni desbloqueó Notion. Capturas/registros personales quedan ignorados y en
la evidencia privada, fuera de Git.

La referencia muestra icono/título, Tag/URL/Favorite/Type, relaciones y un bloque
de código. La nota local equivalente se acreditó por identidad de origen: revisión
6, cuerpo editable y documento con encabezado/listado del importador, párrafo
plano de metadata repetida y código. Ese párrafo ocupaba gran parte del cuerpo.

## Implementación y conservación

- Note normal usa propiedades locales editables Type/Favorite, tags reales,
  URL de origen de lectura y proyectos/personas navegables. Inbox, procedencia
  y snapshot completo están en View details.
- El prefijo Notes repetido exige fuente notion/Notes, marcador exacto, primer
  párrafo plano, tres anclas exactas, nombres admitidos únicos y sin contradicciones.
  Se conserva entero; texto desconocido/ambiguo sigue en el cuerpo. El alcance
  anterior para Goals/Milestones no se amplía a páginas sin esa procedencia.
- Enter/placeholder y estructuras de Note vacía usan BlockEditor compartido.
  Task mantiene las etiquetas previas; Goal/Milestone no reciben estas opciones.
  Sólo un párrafo vacío admite estructuras; listas/encabezados existentes no.
- Baseline privado y backup consistente antes del cambio. Rust, contratos,
  adaptador del editor, locks y TaskPageProperties conservaron SHA-256. Todos
  los bytes CSS anteriores se conservan como prefijo. Sin migración, operación,
  dependencia ni nuevo formato persistente.
- Inspección del mapeo sobre el documento leído: tres bloques de prefijo,
  doce propiedades y código como único bloque visible; recomponer el cuerpo
  original produce el mismo documento completo y no modifica la entrada.
- Lectura canónica posterior: entidad completa, revisión 6, payload y documento
  exactamente iguales a la lectura previa.

## Compilación y revisión manual

`npm run build` aprobado: contratos/iconos y Svelte con 0 errores/0 advertencias;
Vite mantiene el aviso previo de bundle mayor a 500 kB. Bundle aceptado
`index-Dcd4d6ce.js` y CSS `index-Ceq4Qscq.css`.

Se usó la interfaz compilada en Chromium con puente CLI de lectura sobre copia
restaurada. Las mutaciones de entidad se rechazan antes del puente; no se pidió
ninguna. Todos los borradores de demostración fueron descartados.

- La nota importada abre Guardado, con una fila de propiedades y código visible.
  Enter enfoca el cuerpo sin marcar cambios. View details conserva Inbox false
  y las doce propiedades originales.
- Editar Type a Idea, marcar Favorite y escribir en el cuerpo prepara Cambios
  pendientes; undo/redo conserva el flujo. Cancelar navegación al tag mantiene
  el borrador. Contraer/expandir el lateral conserva el mismo nodo de editor.
- Note nueva: ausencias reales, un único grupo de propiedades y tres estructuras.
  Página vacía enfoca, Checklist crea dos checks con cinco IDs únicos, y Nota con
  secciones crea Contexto/Notas. La checklist tiene inputs de 15 px/labels de 18 px.
  El menú `/` funciona y un encabezado vacío ya no ofrece reemplazo por estructuras.
- Task conserva su componente y las tres etiquetas previas; Goal conserva Status
  y Milestone su fecha Date Completed, sin propiedades/estructuras de Note normal.
- Temas claro/oscuro y 420 px revisados. Viewport/documento 420/420, cuerpo y
  código de 380 px. La captura final vuelve a la nota guardada, Reference y
  Favorite false, sin texto de demostración ni estructuras ofrecidas.
- Consola de Noteka: 0 mensajes/errores/advertencias. Carga inicial sin solicitudes
  externas observadas; cero intentos de mutación durante el recorrido.

Los registros iniciales incluyen errores del controlador (snippet con formato
incorrecto, dos nombres accesibles equivocados y una recarga detenida por el
beforeunload de un borrador). Se corrigieron y conservaron. Dos mediciones
iniciales consultaron otro input y una carga aún pendiente: la inspección final
usa input[type=checkbox] y espera el componente de Task. No se atribuyen esos
resultados iniciales a fallos del producto.

## Límites

No se agregaron ni ejecutaron suites. No se ejercitó guardado real, conflicto CAS,
exportación/restauración de un cuerpo nuevo ni IPC/cierre nativos. La recomposición
observada es del adaptador y la revisión UI es del borrador; no acreditan
persistencia. No se capturó ni cerró la aplicación nativa. No se cambió contenido
personal ni se sincronizó Notion. Sin resaltado SQL nuevo, Content, comentarios,
autoguardado, audio o ASR.

## Cierre

Dos requisitos añadidos sincronizados y contrastados: desktop-shell 59→60 y
block-editor 4→5. Los 63 requisitos previos conservaron todos sus bytes; las
otras quince specs y los ocho artefactos de Meeting Notes permanecen intactos.
Archivado 4/4 como `2026-10-24-match-note-detail-reference`.
Validación estricta final: 18/18.
Meeting Notes continúa propuesto 0/19 con sus ocho artefactos intactos.
