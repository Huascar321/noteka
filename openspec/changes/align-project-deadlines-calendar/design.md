# Design

## Context

La consulta `projects_deadlines` existente excluye Done y sin fecha; cambiarla rompería la semántica de clientes existentes. El calendario remoto usa Target Deadline, el campo local `end_date`.

## Goals / Non-Goals

**Goals:** calendario mensual por fecha civil de proyectos no archivados, con Done y sin fecha, y preservación de consultas previas.

**Non-Goals:** importar filtros, automatizaciones o datos remotos; sincronizar Notion; modificar entidades o inventar fechas.

## Decisions

- Agregar `projects_calendar` a `home.query` con todos los proyectos interpretables no archivados, orden estable y paginación previa a presentación; conservar `projects_deadlines` intacta.
- Reutilizar funciones de fecha y superficies del calendario Project, con `page.date` y zona del workspace, y señalar explícitamente páginas aún no cargadas.
- Un día prepara Project Planned con end_date y guardado explícito; errores de fecha se muestran separados sin modificar el original.

## Risks / Trade-offs

- [Vista incompleta] → mostrar aviso mientras exista cursor y permitir Load more.
- [Compatibilidad] → nueva variante aditiva; clientes anteriores siguen usando sus valores conocidos.
