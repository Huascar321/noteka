# Design

## Context

Due ya separa material y contenido con Web Animations API. Los otros overlays mezclan nodos condicionales, `<dialog>` modal y paneles siempre montados con `hidden`. El editor depende de su jerarquía y los CSS de hijos directos; PageTools debe conservar sus buffers.

## Goals / Non-Goals

**Goals:** compartir curvas y tiempos aprobados, mantener estructura/operaciones, y coordinar desmontaje y presentación sin comprimir texto.

**Non-Goals:** refracción nativa, dependencias, cambios de datos, modificar Side/Full peek o disclosures de documentos.

## Decisions

- Extraer tiempos y keyframes de Due a una utilidad común, conservando sus identificadores y cierre existente.
- Integrar los demás overlays explícitamente mediante acción de ciclo de vida y transición Svelte; no observar globalmente ni inferir todos los menús por clases. El contenedor y su texto no se escalan. Una capa de material decorativa manual en top layer reproduce la superficie local durante el morph y evita recortes de overflow. No captura foco/eventos ni añade un segundo blur al contenido.
- La transición se prepara diferida hasta que los dueños posicionen el panel y ejecuten `showModal`. Los owners siguen resolviendo selección, guardias, CAS y devolución de foco. El cierre de Svelte mantiene el DOM visual hasta terminar y lo vuelve inerte; se prueba especialmente el top layer de dialog y su limpieza.
- PageTools conserva las secciones montadas: al ocultarlas, una variante de presencia mantiene sólo su representación visual durante la salida, con `hidden`, `aria-hidden` e inert para semántica/consultas; no se desmonta su contenido. Nav conserva `<details>` y contenido en flujo mediante un adaptador local que demora su cierre visual.
- Los anclajes se pasan como funciones (botón, caret/selección o control original) y se miden al iniciar/cerrar. Sin ancla conectada/visible se usa una forma central de escala moderada. Los diálogos grandes limitan la sobreoscilación al viewport.
- Las entradas/salidas de Svelte son separadas: una salida temprana empieza en su frame capturado y conserva los 200 ms nominales sin aplicar otra vez el progreso de una entrada bidireccional. Search y guardias esperan la salida modal antes de continuar navegación/acciones aprobadas, para evitar que el cleanup restaure foco sobre el nuevo destino.
- `Reduce Motion` elimina los efectos. Contraste/transparencia reducida conservan los fallbacks. Limpieza local al terminar/desmontarse y ante cambios de preferencia; sin reloj permanente ni modificaciones de Rust/SQLite.
- Revisión del usuario sobre el panel vacío: los diálogos y Center peek usan material local detrás del contenido, con overflow temporal y recorte de cada hijo a la geometría del vidrio. El contenido real es tenue desde el inicio y gana claridad sin esperar la expansión; no se duplica ni escala texto. Menús/pickers y Due conservan su receta aceptada. La variante evita que el blur de una capa top layer tape el propio contenido del popup.

## Risks / Trade-offs

- Capa decorativa → aria-hidden/inert/pointer-events none, sin autofocus ni operaciones; en diálogos queda detrás del texto, en menús usa top layer. Verificar ambos motores y no interceptar cancel/backdrop.
- Cambios de foco o salida de componentes anidados → pruebas de Escape, Tab, selección, cierre rápido y paso entre menús; evitar transiciones de descendientes de una superficie ya saliente.
- CSS contextual y overflow → copiar la óptica calculada, conservar la estructura y verificar galerías/material en reposo y transición.
- PageTools oculto con borradores → preservar instancia y consultas por `hidden`; no convertir sus paneles a montaje condicional.

## Migration Plan

Sin migración, cambios de formato u operaciones. El rollback revierte sólo estos hunks y conserva el trabajo local previo. No publicar ni archivar en esta tarea.
