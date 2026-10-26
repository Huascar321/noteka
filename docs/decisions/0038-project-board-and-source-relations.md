# ADR0038 — Tablero de proyectos y relaciones de origen

Fecha: 23/10/2026. Estado: aceptada.

El recorte de Goals pendiente se resuelve el 24/10/2026 en
[ADR0045](0045-goals-as-typed-pages.md), conservando esas páginas como subtipos
tipados. El resto de esta decisión describe su entrega original de Projects.

## Contexto

La presentación genérica de Projects no reproduce el tablero solicitado y sus
avances omiten relaciones perdidas durante la importación. SQLite y los
documentos originales siguen siendo la fuente de verdad; copiar un porcentaje
de Notion no recupera el significado de sus datos.

## Decisión

La UI presenta cinco columnas de 196 px y tarjetas de proyecto, con acciones de
captura y navegación existentes. Rust filtra, ordena y calcula resúmenes antes
de paginar. `progress_tenths` es una proyección derivada; conserva el campo
entero anterior y no persiste porcentajes de origen.

ProjectFields añade procedencia con NoteSource y orden opcional acotado, sin
migración SQL. La procedencia archivada controla Home sin destruir ni archivar
entidades locales. El orden acreditado es un snapshot de Board; no implica
arrastrar tarjetas o sincronizar Notion.

Se reutilizan relaciones existentes: Task→Project, Project→Tag y Note→Project.
Sólo las notas con procedencia Goals validada aparecen como contexto de objetivo
en proyectos; conservan su identidad y cuerpo original. Un dominio Goals queda
pendiente y requiere otro cambio.

La recuperación es explícita y genérica, exige evidencia de identidad, backup,
CAS e informe privado incremental. Los recursos visuales proceden de URL
observadas; los SVG se rasterizan a PNG aislado y se importan como adjuntos.
Los iconos elegidos localmente se conservan. Los documentos nunca se envían como
update y se contrastan IDs y digest documental antes/después.

## Alternativas consideradas

- Guardar porcentajes y contadores de Notion: se descarta porque quedarían
  desactualizados al editar tareas locales.
- Archivar entidades según la fuente: se descarta porque cambiaría el ciclo de
  vida local y el acceso a documentos existentes.
- Crear Goal ahora: se difiere porque exige estados, operaciones y migración
  propios; una referencia a su documento resuelve el alcance solicitado.

## Consecuencias

Hay compatibilidad con payloads anteriores, pero los clientes antiguos de
payload cerrado necesitan actualizarse antes de editar los enriquecidos.
La recuperación no es atómica globalmente y no sustituye una sincronización;
referencias ambiguas o sin documento fuente quedan informadas y omitidas.
El alcance de revisión actual es compilación e inspección visual manual.
