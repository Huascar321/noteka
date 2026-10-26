# Design

## Context

Ver propuesta para motivación. HomeProjectSection tiene Board/Active/Planned; pageMode agrega listas, Deadlines y Completed para Tasks. WorkspacePane ya compone páginas Notes/Tasks con breadcrumb y guardias. home.query conoce relaciones completas de cada página antes del límite de contexto; PeopleActiveProjects sólo cubre proyectos Doing/Ongoing y no sirve para By People completo.

## Goals / Non-Goals

**Goals:** reutilizar filas/tablero, ampliar las proyecciones mínimas y entregar la composición pedida sin reglas duplicadas en Svelte.

**Non-Goals:** importación/sincronización, cambios de datos, calendarios nuevos, rediseño del editor individual, movimiento de relaciones por arrastre o réplica de permisos/compartir de Notion. Deadlines conserva la lista existente y sus reglas. Las vistas auxiliares se presentan como agrupaciones locales dentro de Projects.

## Decisions

1. ProjectsPage monta un HomeReader de cien proyectos con acumulación y ReadQueue(2), projects_active inicial. Reutiliza el patrón de Notes/Tasks, descarte tardío/dispose y actualización ⌘R/Ctrl+R/F5 sólo cuando está activa y disponible; diálogos/composición/repetición no inician lectura. WorkspacePane omite el listado genérico y conserva breadcrumb/openHome/createHome.
2. HomeProjectSection añade standalone opcional además de pageMode. Sólo standalone muestra icono/título grande, Nav, New Project, On Hold y agrupaciones auxiliares; Home y Projects en Tasks mantienen sus variantes. Nav permite volver a Active/Board y By Tag/People selecciona consultas reales. Las agrupaciones se presentan en listas compactas plegables, con estado por fila; sus enlaces abren tag/persona reales. El tablero por estado conserva arrastre/CAS sin ampliarlo a las relaciones.
3. home.query incorpora projects_on_hold/projects_by_tag/projects_by_people. Las nuevas vistas usan selección y orden de proyectos, exclusión de archivo y el snapshot/cursor existentes. By Tag/People usa contexto completo antes de truncar a cincuenta referencias, con grupo por entidad relacionada y sin relación. HomeGroup añade related_entity opcional, omitido cuando no existe, para mantener icono/ID/título sin depender del contexto truncado. Grupos ordenados por nombre/ID; miembros por estado/board_order/título/ID; cuentan apariciones de página. No se elevan caps.
4. Crear en un grupo de estado reutiliza el estado elegido. En un grupo por relación prepara Planned con ContextLink tag/person de la identidad del grupo; la operación de creación existente guarda vínculo y entidad de forma atómica cuando el usuario confirma. El grupo sin relación crea Planned sin vínculo. Top New Project/New conservan Planned.
5. Se consultó únicamente la estructura/configuración de páginas/vistas Notion para acreditar By Tag/People y On Hold. El conector omite botones de Nav; los controles de captura se basan en la imagen y acciones locales existentes. No se leen cuerpos de registros, cambian propiedades remotas ni se copian nombres personales a fixtures/documentación.

6. On Hold usa Pause del catálogo local AnimateIcons fijado, agregado mediante selección canónica y generador. No hay nueva dependencia, animación ni modificación de iconos previos.

## Risks / Trade-offs

- Los datos y orden de la referencia pueden diferir del snapshot local → conservar cálculos y orden persistente local, sin igualar datos con una imagen.
- Una entidad pertenece a varios grupos → IDs únicos en items, pertenencia explícita por grupo y cobertura de página en opciones.
- Componentes compartidos → props opcionales, variantes previas por defecto y cambios de estilo acotados.
- Proyección nueva con datos inválidos → mantener unavailable/error/caps; no reparar originales ni usar respuesta vieja como vigente.

## Migration Plan

Sin SQLite/payload nuevos. Baseline privado antes de editar; compilar CLI offline, generar/comparar contratos y compilar frontend. Inspección visual manual sobre copia local de sólo lectura en claro/oscuro/estrecho, sin suites ni nueva aceptación Tauri. Reversión mediante archivos del baseline; no requiere reemplazar el workspace. Registrar la decisión de agrupación en ADR0044 y guías/plan.
