# Spec Delta

## ADDED Requirements

### Requirement: Proyecciones de Goals y Milestones compartidas

home.query SHALL ofrecer Goals por actividad, plazo, tag y logrados, y Milestones
completados/todos con filtro opcional por Goal. MUST filtrar archivo antes de
paginar, ordenar con desempates título/ID, calcular contexto y grupos del mismo
snapshot y conservar páginas 1–100, caps 10.000 candidatos/fuentes, 50.000 vínculos,
200 grupos y cursores ligados a workspace/vista/día/límite/Goal. By Tag SHALL
mostrar objetivos Active por sus tags y List hitos con fecha completada.

#### Scenario: Vistas y grupos
- **WHEN** se consultan Goals con tres estados y relaciones o Milestones con y sin completar
- **THEN** actividad/plazo mantienen Active/Dream/Achieved incluidos vacíos,
  By Tag agrupa los Active, Achieved filtra logrados y List filtra completados;
  los grupos referencian sólo IDs de la página y los hitos de un Goal se pueden consultar

#### Scenario: Cursor o referencia incompatible
- **WHEN** se cambia workspace, vista, día, límite o Goal de un cursor, o se
  usa el filtro de Goal en una vista que no lo admite
- **THEN** se rechaza la lectura sin mezclar páginas, mutar datos o ampliar cuotas

### Requirement: Progreso y actividad derivados de objetivos

Rust SHALL calcular progreso como hitos activos completados/total, redondeado a
una décima porcentual. Sin hitos SHALL informar ausencia de progreso y un hito
no interpretable SHALL invalidar el resumen afectado. MUST mantener estado de
Goal y finalización de hitos independientes. Actividad SHALL combinar snapshot
importado acreditado con ediciones locales posteriores del Goal, hitos,
proyectos y sus notas/tareas, distinguiendo revisiones de recuperación.

#### Scenario: Hitos, ausencia y estado
- **WHEN** cambian hitos relacionados o no existen hitos activos
- **THEN** la próxima lectura recalcula el progreso completo sin limitarlo a
  filas visibles ni modificar automáticamente estado o fecha de logro; sin hitos no inventa cero

#### Scenario: Snapshot y edición local
- **WHEN** se recupera metadata con actividad de origen y luego se edita una fuente relacionada
- **THEN** la recuperación conserva la actividad observada y una revisión
  posterior participa con su timestamp local; no se promete sincronización con Notion
