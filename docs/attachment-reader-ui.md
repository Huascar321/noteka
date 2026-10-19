# Lectura explícita de generaciones locales

Cambio `add-local-attachment-reader-ui`, con aceptación integrada aprobada. El controlador/panel
aprobaron verificación aislada y revalidación conjunta de las UIs,
manteniendo Rust/corpus/contratos de preparación sin cambios. Esto no entrega OCR, PDF/Excel genéricos ni toda 5a.

El lector permite ID de trabajo y tipo explícitos. Usa únicamente
`extraction.fragments`, `extraction.pdf_fragments` o `extraction.xlsx_cells`,
por páginas de veinte, sin consultas al montar/editar/activar ventana y sin
requerir selección en Search. Pedir siguiente reemplaza, no acumula; inicio
vuelve a leer sin cursor. Lectores Rust revalidan fuente/derivado completos;
error retira texto/cursor anteriores y no repara ni ejecuta fallback.

UTF-8 preserva texto/bytes/líneas originales. PDF conserva texto por página,
localizadores derivados, manifiesto con no_text/scope/limitaciones. No es render,
geometría, tablas u OCR. XLSX muestra celdas dispersas con hoja/A1/ordinal, tipo
guardado/fórmula/caché separados y manifiesto completo: visibilidad, rangos,
merges, sistema de fechas y estilos. Números y date_iso siguen cadenas léxicas,
sin redondear, convertir seriales, inferir encabezados o recalcular fórmulas.

`covers_generation` se presenta como dato del núcleo, junto con complete e
identidad/versión/configuración/fuente. Un manifiesto completo no hace completa
una página parcial. Resultado de intento anterior es explícito. Cero items no
significa ausencia de información visual ni OCR realizado.

Lifetime por criterios/contexto y un cupo físico entre montajes. Cancelar espera
no cancela IPC iniciado, cuyo resultado viejo se descarta; una llamada pendiente
en cola no llega a ejecutarse. No hay caché persistente de contenido ni remotos.
El editor y sus guardias permanecen autoridad de la edición pendiente.

Prueba de cliente independiente:

```sh
node --experimental-strip-types --test tests/attachment-generation-reader.mjs
npm run check
```

Ese comando prueba el cliente; la [verificación integrada](verification-attachment-reader-ui.md)
registra por separado las pruebas UI. Ninguna sustituye IPC nativo ni el original administrado.
