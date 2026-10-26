# Tags en Home

La [página lateral Tags](tags-page.md) usa composición propia con icono/título,
Nav, New Tag y cinco vistas; inicia Recent y añade Table como presentación A-Z.
Las reglas siguientes corresponden a las proyecciones compartidas. Home y el
bloque incrustado en Notes conservan sus cuatro vistas.

El [detalle de Tag](tag-detail.md) presenta Type/Favorite/Note Count y contenidos
del contexto. Conserva las cuatro clasificaciones y usa el resumen Rust para el
contador; ausencia de resumen no se transforma en cero.

Types abre la sección, con Area naranja, Resource violeta y Entity verde. Topic
conserva un grupo independiente cuando hay registros; no se convierte en Entity.
Las otras vistas son Recent, Fav. y A-Z. New prepara un Resource, la creación de
grupo conserva su clasificación y crear en Fav. prepara la marca de favorito.
La captura mantiene las guardias existentes y no se guarda automáticamente.

Las filas muestran icono, título, última nota y cantidad. Los originales son
adjuntos locales o iconos nativos; abrir Home no descarga imágenes. La última
nota y el tag abren sus registros originales. Las ventanas estrechas distribuyen
la metadata debajo del título sin overflow global.

## Reglas de datos

Rust filtra los tags archivados localmente o en origen antes de paginar; los
archivados sólo en origen siguen accesibles en listados generales. Fav. usa la
marca explícita. A-Z ordena por título Unicode y Types por clasificación/título,
con desempate por ID. No se promete collation local.

El contador usa notas relacionadas únicas y excluye archivo local y de origen.
La última nota se elige por su timestamp original validado cuando existe y por
modificación local en las demás, interpretando RFC3339 como instantes. El orden
de importación o recuperación de iconos no determina qué nota es la última.

Recent combina actividad importada acreditada y el máximo timestamp de las
notas relacionadas no archivadas. Ese cálculo se hace antes de paginar, dentro
del snapshot y límites de Home. El snapshot importado no implementa todas las
fórmulas de actividad de Notion ni una sincronización de proyectos/tareas remotos.
Por eso su orden puede diferir del de Notion tras cambios no importados.

TagFields admite favorito, procedencia y actividad de origen opcionales.
La actividad es RFC3339 acotada y se valida en Rust. El detalle permite
clasificación/favorito y conserva la procedencia/actividad de lectura. Los
payloads antiguos siguen siendo válidos, sin migración SQL. Clientes anteriores
con payload cerrado deben actualizarse antes de editar los enriquecidos.

## Recuperación

El helper `scripts/recover-notion-tag-metadata.py` verifica las quince filas
CSV/manifiesto y las propiedades iniciales de sus páginas originales. Recupera
189 vínculos Note→Tag: 181 nuevos y ocho existentes. Los trece tags activos
resumen 146 apariciones de notas no archivadas; Personal tiene 35 referencias y
excluye una archivada, por lo que muestra 34. No se copian contadores de Notion.

Se añadieron diez asignaciones de icono acreditadas por la referencia. Se
conservaron los iconos elegidos y los originales. Backup nuevo, CAS e informe
privado incremental; los 1.802 IDs y todos los documentos quedaron intactos.

Una dependencia de metadata tenía dos títulos de nota mayores a 240 caracteres
clasificados como Pages por truncar el índice CSV. El helper de notas ahora usa
el título completo y ofrece `--repair-truncated-titles`, limitado a identidades
originales inequívocas y a esa procedencia heredada. Se corrigieron esas dos
notas con otro backup y sin cambiar sus cuerpos. Clips pasa de 50 a 52;
Notes permanece en 132 y All en 1.172 en el snapshot revisado.

Los informes, fuentes, imágenes y backups quedan fuera de Git. No hay atomicidad
global ni restauración automática sobre cambios posteriores; una reversión debe
considerar las revisiones actuales. No se recuperaron jerarquías, People o Goals
adicionales como parte de este cambio.
