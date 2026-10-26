# Design

## Context

Ver propuesta para la motivación. HomeNoteSection ya ofrece ocho vistas, metadatos y captura contextual; HomeTagSection ofrece cuatro vistas y resúmenes locales. TasksPage ya compone dos secciones mediante HomeReader y ReadQueue. WorkspacePane conserva navegación, breadcrumb y guardias de borrador.

## Goals / Non-Goals

**Goals:** reutilizar esa composición para la entrada Notes y adaptar sólo el encabezado y la presentación contextual de Tags.

**Non-Goals:** cambios de reglas Rust, migraciones, importación/sincronización, nuevos tipos, grabación Voice, permisos/compartir remotos o rediseño del editor individual. No se copian títulos ni fechas del screenshot como datos ficticios.

## Decisions

1. NotesPage monta dos HomeReader con cola física compartida de dos lecturas: notes_library y tags_recent, diez filas por página y acumulación sin duplicados. Cambiar vista/actualizar utiliza la invalidación existente; desmontar dispone ambos lectores. Alternativa descartada: duplicar filtros/orden en Svelte o usar entity.list, porque perdería procedencia y reglas de biblioteca de Rust.
2. HomeNoteSection recibe pageMode opcional, manteniendo su composición Home por defecto. La variante tiene icono/título grande y Nav con atajos a vistas y desplazamiento a Tags. Reutiliza todas las filas y semillas de creación. NotesPage comparte los estilos de página existentes y añade únicamente ajustes de Notes.
3. HomeTagSection recibe pageMode opcional para la sección integrada: filas sin contador de notas adicional, encabezado enlazado al listado general y New page textual de ancho completo. Home conserva contador, agrupación y botón con borde. Las vistas/consultas/resúmenes no cambian.
4. WorkspacePane omite entity.list para la página Notes y monta NotesPage sólo en overview. Sus callbacks usan openHome/createHome/navigate; el breadcrumb compartido sigue siendo su propietario. Actualizar vuelve a leer ambas secciones; ⌘R/Ctrl+R/F5 sólo actúa en la página activa disponible y bloquea recarga ante diálogos/composición/repetición.

## Risks / Trade-offs

- La referencia contiene fechas/datos de otro momento → presentar el snapshot local y su procedencia, sin sincronización automática.
- Las pestañas exceden un viewport estrecho → conservar desplazamiento interno y metadatos que pasan a segunda línea.
- Cambios en componentes compartidos → props opcionales con valores Home actuales y estilos acotados a la página Notes.

## Migration Plan

Sin cambios persistentes ni contratos. Baseline privado de archivos afectados para reversión; conservar cambios previos. Compilar frontend e inspeccionar manualmente claro/oscuro y ancho estrecho sobre una copia local con puente de sólo lectura. No ejecutar suites ni atribuir esa inspección a IPC/cierre Tauri. La reutilización de la arquitectura de ADR0043 se documenta en la guía y el plan general.
