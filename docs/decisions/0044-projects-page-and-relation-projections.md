# ADR0044: página Projects y proyecciones por relación

Fecha: 24 de octubre de 2026. Estado: adoptada.

## Contexto

El usuario pidió la página Projects de su referencia Notion. El escritorio ya
tenía filas/tablero en Home/Tasks, pero la entrada lateral era un listado
genérico. Las configuraciones leídas de Notion acreditan On Hold y navegación a
By Tag/By People. PeopleActiveProjects sólo incluye Doing/Ongoing y no representa
la agrupación completa por persona.

## Decisión

Reutilizar el patrón de composición de ADR0043 en ProjectsPage y las variantes
opcionales de HomeProjectSection. Active inicial, seis pestañas, breadcrumb y
guardias existentes. Deadlines conserva su lista local por fecha.

Agregar tres vistas compartidas de home.query: projects_on_hold, projects_by_tag
y projects_by_people. Rust selecciona/ordena proyectos y agrupa por relaciones
explícitas dentro del mismo snapshot de lectura. HomeGroup.related_entity es
opcional y permite abrir/presentar el original aunque el contexto de una fila
esté truncado. Sin vínculo forma un grupo explícito. Las agrupaciones cubren la
página y cada proyecto conserva su única identidad, aun con varias apariciones.

By Tag/People se presentan como listas plegables dentro de Projects, con estado
por fila; no agregan un motor de relaciones en el cliente. Crear en un grupo
reutiliza ContextLink y la creación atómica existente, sin guardar automáticamente.
El movimiento por estado conserva exclusivamente el Board y su CAS/receipt.

## Alternativas

- Agrupar en Svelte desde contexto truncado: descartada por pertenencias
  incompletas y reglas diferentes entre clientes.
- Reutilizar PeopleActiveProjects: descartada por excluir otros estados y
  personas fuera de la biblioteca de contactos de esa vista.
- Nueva tabla o estado persistido para cada agrupación: innecesario; relaciones
  y entidades existentes siguen siendo la fuente de verdad.

## Consecuencias

Sin migración, aumento de cuotas, nuevas operaciones o dependencias. La extensión
de contratos es aditiva; related_entity se omite en grupos anteriores. Las
opciones de vista explican cobertura y apariciones. Home/Tasks conservan sus
variantes y el editor sigue abriendo originales.

Pause se agrega al catálogo estático del paquete AnimateIcons fijado mediante
selección canónica/generador. Los SVG anteriores, iconos laterales y licencias
se conservan. Compilación e inspección manual sobre copia local no sustituyen
aceptación nativa ni pruebas completas de fallos/concurrencia.
