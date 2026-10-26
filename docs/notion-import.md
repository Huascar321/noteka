# Importación del export de Notion

Fecha de ejecución: 23 de octubre de 2026.

Este documento describe una importación local del export Markdown/CSV de Notion
en el workspace por defecto de Noteka. No es una promesa de paridad total con
Notion ni un importador incremental reutilizable sin revisar el contrato.

## Entrada y workspace

- Export de origen: carpeta local `Export-6c623508-…` fuera del repositorio.
- Destino: `$HOME/Library/Application Support/Noteka/workspace`.
- Copia del workspace anterior a la importación final:
  `$HOME/Library/Application Support/Noteka/workspace-notion-v2`.
- Manifiesto: `$HOME/Library/Application Support/Noteka/notion-import-manifest.json`.
- Script reproducible: [`scripts/import-notion-export.py`](../scripts/import-notion-export.py).

Comando usado:

```sh
python3 scripts/import-notion-export.py \
  --export "$HOME/Downloads/Export-6c623508-efe6-4a25-a2e0-5b61ab7c4a44" \
  --workspace "$HOME/Library/Application Support/Noteka/workspace" \
  --cli target/debug/noteka
```

## Resultado

| Dato | Cantidad |
|---|---:|
| Páginas Markdown | 1.103 |
| Notas creadas (páginas + filas sin página) | 1.172 |
| Tareas | 578 |
| Proyectos | 20 |
| Tags | 15 |
| Personas | 17 |
| Adjuntos importados | 147 |
| Fallos de importación | 0 |

Los 147 adjuntos incluyen 124 imágenes locales (PNG/JPEG/GIF) y documentos
auxiliares del export (PDF, XLSX, DOCX, SQL, MP4). Las imágenes referenciadas en
Markdown se convierten en bloques `image` con `attachment_id`; los archivos
locales enlazados se convierten en bloques `file`. Se creó además una nota
`Notion · Archivos` con el catálogo de adjuntos.

La importación conserva:

- títulos y cuerpo Markdown de las 1.103 páginas;
- encabezados, listas, checklist, citas, código, reglas e imágenes locales;
- propiedades CSV como una sección `Propiedades de Notion` al inicio de la
  página correspondiente;
- entidades tipadas para las bases Tasks, Projects, Tags y People;
- notas para filas de otras bases que no tenían una página Markdown propia;
- fechas civiles de tareas/proyectos cuando el formato del export lo permite.

## Límites conocidos

- **Jerarquía de páginas:** no se reconstruyó el árbol de páginas padre/hijo en
  el modelo de entidades. Todas las páginas viven en la lista plana de Notes.
- **Iconos y portadas de Notion:** el Markdown no contiene el icono ni la portada
  personalizados de cada página. Se usa el icono de catálogo por defecto del tipo y el
  usuario puede elegir otro desde la página.
- **Relaciones Notion:** el CSV exportado no incluye todas las columnas de
  relación; no se inventaron relaciones ausentes ni se duplicaron proyectos.
- **Iconos remotos:** los `<img>` remotos de iconos de Notion se descartan como
  decoración; no se descargan URLs remotas durante la importación. Las imágenes
  locales sí se copian.
- **Filas sin título:** se conservan con un título derivado de su primera
  propiedad significativa o con `Sin título (Notion · base)`.
- **Duplicación deliberada:** una fila de base puede existir como entidad tipada
  y como nota de página. Es la forma de conservar contenido y propiedades sin
  perder ninguna de las dos vistas.

## Verificación

- El manifiesto reportó 1.802 entidades creadas y cero fallos.
- SQLite contiene 1.172 notas, 578 tareas, 20 proyectos, 15 tags y 17 personas.
- La carpeta `attachments` contiene 147 originales administrados.
- Se comprobó una nota con imágenes locales (`Ana flowers`) y una página Web Clip
  (`Book Data Transfer Tool`) con formato semántico, sin HTML crudo.
- `npm run check`, `npm run build`, `cargo test -p noteka-core`,
  `npm run test:headless` y el recorrido nativo pasaron después de los cambios.

## Interfaz

- La portada de Home usa una animación ASCII de tiburón; el color sigue el tema
  (blanco sobre oscuro, negro sobre claro) y el espacio vertical es de 300 px.
- Los iconos de navegación y entidades usan el catálogo SVG local, no emojis. Ver
  [iconos](icons.md) y [ADR 0034](decisions/0034-entity-visual-metadata.md).
- Las listas usan iconos por tipo y las páginas permiten elegir icono y color.

## Pendiente

- Reconstruir jerarquía, portadas e iconos originales de Notion requeriría un
  export adicional o una consulta autenticada a la API de Notion.
- El Google Doc sintético `Noteka Preview Test Doc` ya fue eliminado el
  23/10/2026 con autorización explícita; no se añadió borrado de Drive a la API
  del producto.
