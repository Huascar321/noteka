# Design

## Context

Ver proposal.md. NoteFields es estricto y el payload original vive en SQLite;
entity.update fusiona parches y valida mediante Rust/CAS. PageTools tiene un menú
de Note normal y WorkspacePane conserva borrador/guardia/editor. Bloquear es una
preferencia de interfaz contra accidentes, no una nueva barrera de permisos.

## Goals / Non-Goals

**Goals:** estado durable en datos originales, apertura temporal por instancia,
edición completa protegida y preservación de documento/undo/guardias.

**Non-Goals:** ACL, contraseñas, bloquear clientes CLI/MCP, migraciones SQL,
bloquear registros de colecciones referenciadas o ampliar Goal/Task/Milestone.

## Decisions

1. `NoteFields.page_locked: Option<bool>` omitido cuando no existe; no se añade
   un default serializado a Notes anteriores. Se reutiliza entity.update con
   parche de payload y revisión esperada, sin enviar documento ni título.
2. La apertura temporal se identifica con el ID de la Note en WorkspacePane;
   no va al payload, localStorage ni al documento. ClearSelection la descarta;
   un guardado y cambiar la presentación de la misma instancia la conservan.
3. Bloquear o re-bloquear un borrador usa la guardia existente. Tras Discard se
   repone el documento guardado antes de aplicar el nuevo modo. El estado durable
   sólo cambia después del resultado confirmado; errores mantienen el modo previo.
4. Se deshabilitan título/cuerpo/icono/campos y acciones de relaciones/recursos,
   además de handlers de mutación y guardado. Se conserva lectura/copia/navegación,
   exportación y lectores de adjuntos. No se agrega una capa sobre el documento
   que bloquee selección o apertura de referencias.
5. PageTools añade acciones checkbox accesibles sin controles interactivos
   anidados. El switch expresa la preferencia, incluso durante edición temporal.
   El indicador a la derecha del breadcrumb usa Locked/Re-lock y candados.
6. Los contratos se regeneran desde Rust; no hay operaciones ni dependencias
   nuevas. ADR0054 define compatibilidad y recuperación antes de implementar.

## Risks / Trade-offs

- Clientes viejos estrictos → lectura estructurada sigue disponible; parches
  completos que desconocen el campo se rechazan sin descartarlo. No se promete
  editar payloads nuevos con binarios viejos.
- Otra instancia modifica la revisión → CAS evita pisar contenido; se conserva
  el error/buffer y refresh/reapertura recupera el estado confirmado.
- Olvidar un control de edición → guardas compartidas más revisión headless de
  título, contenido, propiedades, relaciones, recursos, atajos y clic inferior.

## Migration Plan

Campo opcional dentro del JSON existente: no DDL ni incremento de versión
SQLite. Ausencia/null equivale a desbloqueada. No se recorren ni reescriben Notes.
Backup/restore/exportación conservan el campo automáticamente; se prueban.
Rollback conserva el JSON y permite recuperarlo; no se elimina una clave para
forzar compatibilidad. Documentos v1/v2/v3 y propiedades desconocidas se preservan.
