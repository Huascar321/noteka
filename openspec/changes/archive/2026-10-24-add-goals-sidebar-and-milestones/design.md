# Design

## Context

Existen tres Goals reales, su plantilla y dieciséis Milestones importados como
note con procedencia por ID. Cinco vínculos Goal→Project ya están recuperados;
ocho hitos tienen Goal y ocho no. La referencia tiene cuatro vistas; By Tag filtra
Active y List de Milestones sólo fecha completada no vacía, actualmente sin filas.
La fórmula leída usa hitos completados/total, no tareas de proyecto.

## Goals / Non-Goals

**Goals:** comportamiento útil sobre identidades originales, reglas compartidas,
edición/captura protegida y presentación de referencia. Ver propuesta/deltas.

**Non-Goals:** nueva familia de tablas/EntityKind, sincronización remota,
Action Center/automatizaciones de Notion, arrastre de Goals o suites nuevas.

## Decisions

1. Extender NoteFields con GoalFields/MilestoneFields opcionales excluyentes;
   no cambiar EntityKind ni el documento. ADR0045 actualiza el recorte pendiente
   de ADR0038: la identidad existente resuelve Goal como subtipo, sin reconstruir
   entidades ni forzar una migración. Generic CRUD, lotes y CAS siguen siendo
   operaciones canónicas. Rust valida fechas/estados y referencia a Goal, observa
   la dependencia en MutationContext y rechaza quitar un Goal aún referenciado.
   Relaciones Note→Project/Tag existentes; Goal admite un tag directo.
2. Módulo de proyección Goals separado, despachado por home.query con seis vistas
   aditivas. HomeQuery.goal_id opcional sólo para milestones_all. Lectura sin
   documentos, transacción única, caps/cursor/contexto acotados. Grupos por estado
   y tags conservan IDs originales; el resumen usa todos los hitos activos del Goal.
   By Deadline ordena por plazo con ausentes al final y conserva actividad visible.
3. Goal conserva snapshot opcional `{at, revisions}` de actividad. Las revisiones
   incluyen Goal, hitos, proyectos y sus notas/tareas ordinarias al recuperar;
   la recuperación misma no equivale a actividad. Se usa el timestamp observado
   y después el máximo updated_at de fuentes nuevas o revisiones posteriores.
   Sin snapshot se usa metadata original disponible/local. UI de origen observada
   con precisión de minuto y zona explícita; no se inventa precisión de segundos.
   Fuente faltante/metadata inválida se informa. No se implementa fórmula universal.
4. Sidebar usa trophy existente y su definición diferida mediante generador.
   Section lógico goals no reemplaza kind note: una función de ruta reconoce
   payload tipado para breadcrumb, tabs y apertura desde cualquier contexto.
   GoalsPage/HomeGoalSection reutilizan HomeReader/cola/captura. Milestones global
   List es completados; el detalle de Goal consulta milestones_all con goal_id.
   Selector de Goal por título con paginación. Note Library excluye nuevos subtipos;
   All y búsqueda conservan acceso a sus páginas canónicas. La presentación
   importada reconoce el párrafo duplicado de propiedades de Goals/Milestones
   sólo tras el marcador canónico, una línea exacta y nombres conocidos; conserva
   bloques originales en el prefijo/Details y no cambia el documento.
5. Recuperación genérica desde metadata privada: resolver por record_id original,
   verificar esquema/identidades, respaldar y aplicar campos faltantes con CAS;
   relaciones idempotentes y snapshot al final con revisiones vigentes. No convertir
   plantilla ni copiar progreso. Contrastar todos los IDs/documentos antes/después.

## Risks / Trade-offs

- Clientes antiguos de payload cerrado → actualizar contrato antes de editar;
  título/lectura y documentos siguen preservados.
- Snapshot de actividad con fuentes acotadas → documentar precisión y alcance;
  una edición local posterior se diferencia de la recuperación por revisión.
- Recuperación incremental → informe por mutación, CAS, backup y no reintento forzado;
  rollback debe considerar cambios posteriores del usuario.
- Nuevas reglas sobre páginas → revisión de código, compilación e inspección manual
  acotada sobre copia; sin atribuir pruebas nativas o suites no ejecutadas.

Se reconcilian tres requisitos vigentes: menú de siete destinos, Goals real en Home
y contexto de proyectos con Goals tipados; se conservan sus escenarios heredados.

## Migration Plan

SQLite conserva versión 20. Campos opcionales compatibles con páginas anteriores,
contratos generados y SDK actualizados. Baseline y backup canónico privados antes
de modificar fuente/datos; recuperación explícita por operación pública. Reversión
de metadata sólo con revisiones vigentes y respaldo, sin sustituir documentos ni
restaurar sobre ediciones posteriores. Los originales permanecen recuperables.
