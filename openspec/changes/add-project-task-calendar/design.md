# Design

## Context

La referencia usa calendario mensual por Due con anterior, Today, siguiente y un acceso a elementos sin fecha. El detalle local obtiene páginas de relaciones mediante `workflow.query` y las acumula; los datos pueden ser parciales.

## Goals / Non-Goals

**Goals:** navegación civil en la zona del workspace; presentar tareas cargadas, sin fecha o con fecha inválida, y abrir o preparar originales sin writes implícitos.

**Non-Goals:** arrastre, recurrencias, relaciones de proyectos padres, prometer cobertura completa con cursor pendiente o modificar reglas de dominio.

## Decisions

- Reutilizar las tareas y el cursor del detalle, incluyendo Done, y aclarar cuando quedan relaciones por cargar. Ordenar títulos de forma estable.
- Fechas civiles se tratan como fechas, sin conversión horaria. El mes inicial y Today se calculan con la zona del workspace; límites 0001–9999 se protegen.
- Añadir acceso discreto y teclado para cada día, que prepara un borrador con `due_date` y el contexto de Project existente; sin persistir antes de Save.
- Conservar List/Board, guardias y movimientos actuales y probar anchos/temas en ambos motores.

## Risks / Trade-offs

- [Relaciones parciales] → aviso y Load more existentes, sin llamar vacía a una base completa.
- [Campo desconocido/fecha inválida] → mostrar en grupo separado y conservar la entidad, no descartar ni corregir datos.
