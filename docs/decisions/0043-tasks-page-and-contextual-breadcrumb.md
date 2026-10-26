# ADR 0043: página Tasks y navegación contextual compartida

Fecha: 24 de octubre de 2026. Estado: aceptada.

## Contexto

La navegación Tasks todavía montaba el listado genérico, mientras Home ya tenía
consultas, relaciones y filas apropiadas. El usuario pidió la composición Tasks
con Projects debajo, luego un breadcrumb común y pies sin diagnósticos técnicos,
con acciones textuales que ocupen toda la fila.

## Decisión

TasksPage reutiliza HomeReader/HomeTaskSection/HomeProjectSection y las guardias
de WorkspacePane. Today inicia con diez tareas; Projects inicia Active con hasta
cien proyectos por consulta. Las lecturas tienen cola acotada, identidad propia
y descarte tardío; Load more acumula IDs y grupos sin duplicarlos. Home comparte
esa acumulación para sus cinco secciones.

Rust incorpora projects_deadlines/projects_completed a home.query. Deadlines
selecciona proyectos no Done con end_date válida y orden civil ascendente;
Completed selecciona Done. Archivo local/de origen, límites, snapshots y
cursores se conservan. project_end_date es una proyección opcional del campo
persistente existente, no una fecha derivada de tareas.

PageBreadcrumb representa Home → sección canónica → registro, o Home → sección
al navegar una página. Usa iconos propios y arte local existente; los antecesores
usan navigate y sus guardias. El título vivo es accesible completo y puede
truncarse visualmente. No se incorpora el nivel Databases de Notion.

Los contadores de consulta quedan en Opciones de vista. Los pies no muestran
contadores/fechas ni el aviso técnico final de Home. Load more aparece sólo con
cursor y ocupa toda la fila; New page/New task sin borde ocupan su contenedor.
Las capturas y tarjetas con borde mantienen sus estilos. La captura prepara un
borrador y no lo guarda automáticamente.

## Persistencia y recuperación

No se modifica SQLite ni el payload de entidades. Los contratos de lectura se
regeneran con el frontend; no hay operaciones nuevas ni dependencias. Los datos
originales y relaciones existentes son la fuente de verdad. Baseline y evidencia
viven fuera de Git; revertir los archivos de este cambio no requiere restaurar
una base ni reemplazar ediciones posteriores del usuario.

## Alternativas

Duplicar filtros mediante entity.list produciría reglas de negocio divergentes.
Inferir plazos desde tareas o copiar porcentajes de un screenshot produciría
valores ajenos al modelo local. Un breadcrumb basado en historial podría atribuir
la sección de origen al registro equivocado; se usa su tipo canónico.

## Consecuencias

Los resultados pueden diferir de un snapshot de Notion por fechas/estados
locales, y Recurring/Active Content siguen deshabilitadas. El tablero conserva su
límite de movimiento de cien proyectos. Compilación e inspección manual headless
no equivalen a aceptación de IPC/cierre nativo ni a regresión automatizada.
