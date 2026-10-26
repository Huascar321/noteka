# Design

## Context

Hay una sola instancia fuente de Más opciones en WorkspacePane, compartida por todos los detalles. Contiene exportación, recursos externos y preparación/lector de adjuntos para note. La referencia Project se identificó por el vínculo de la Note ya revisada, se abrió/capturó con Playwright y coincide con el ID de origen local; el resumen Rust acredita el avance desde las tareas locales. Capturas y datos personales permanecen fuera de Git.

## Goals / Non-Goals

Ver proposal.md. La entrega mantiene operaciones y condiciones actuales; no crea contenido enriquecido de Project ni copia vistas Board/Calendar inexistentes para tareas. No recupera datos ni transfiere contenido a proveedores.

## Decisions

1. **Paneles compartidos y persistentes.** PageTools recibe grupos disponibles y snippets. Botones nativos con tooltip y aria-controls/expanded; paneles inline ocultos con hidden y Escape devuelve foco. Permanecen montados para conservar formularios/lectores/operaciones al cerrar, como antes con details. Al cambiar de entidad se limpia la selección. Notes muestra tres grupos; exportación/recursos no disponibles quedan deshabilitados con razón, otros tipos omiten Adjuntos. Las subacciones continúan dentro del panel. Alternativa descartada: un icono por operación o desmontar lectores al cerrar.
2. **Cabecera propia.** ProjectPageProperties representa payload.state/end_date y resumen de home.query/projects_board (hasta 100 proyectos, cuota existente). Un original fuera de esa página muestra resumen no disponible; error permite reintentar. La UI sólo convierte unidades de décimas para presentar, sin calcular progreso. EntityFields secundario conserva inicio, propósito y procedencia. Portada con gradiente decorativo, sin inventar una URL persistente ni descargar la portada de la fuente; icono original local.
3. **Listas desde contexto canónico.** ProjectContents agrupa para presentar entidades devueltas por workflow.query/context, conserva un cursor acumulativo común y separa carga/error/ausencia parcial. IDs/títulos no se duplican; estados no reconocidos tienen grupo propio. Goal/Milestone y otras entidades de contexto permanecen en relaciones de cabecera. Captura/open reutilizan newEntity/openEntity y guardias. Sin nuevos filtros de negocio ni proyecciones de progreso en Svelte; sin aparentar Board/Calendar de tareas.
4. **Lecturas tardías.** Generaciones y workspace/ID impiden publicar respuestas de otro detalle; errores de resumen/contexto son independientes. Se conserva paginación y no se reemplaza un borrador local al cambiar panel o recibir datos.

## Risks / Trade-offs

- Página de contexto parcial → mensaje explícito y un cursor común para continuar; no afirmar total vacío de Notes/Tasks.
- Proyecto ausente de Board/primeros 100 → resumen no disponible, nunca 0%; no aumentar cuotas ni recorrer toda la biblioteca sólo para un detalle.
- Panel oculto con operación iniciada → montaje conservado y advertencias/estado anteriores; abrir no llama operaciones ni inicia Picker.
- Estado del formulario frente a datos canónicos → guardado explícito y CAS existentes; revisión en copia con writes bloqueados no acredita persistencia/IPC nativo.

## Migration Plan

Sin migración ni cambios de contratos/Rust/documentos/dependencias. Baseline y backup canónico previos fuera de Git. Revertir interfaz no requiere restaurar datos. Conservar Meeting Notes 0/19 y cambios locales previos; no tocar otros proyectos/procesos.
