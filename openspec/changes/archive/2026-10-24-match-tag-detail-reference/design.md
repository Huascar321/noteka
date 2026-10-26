# Design

## Context

La referencia se identificó desde el vínculo Tag de la Note/Project ya revisados y se abrió/capturó en Notion sin edición. Coincide con la identidad de origen local. Type/Favorite/Note Count y Notes/Clips definen su jerarquía; People/Sub-Tags/Goals de la fuente no autorizan inventar relaciones locales. El detalle actual mezcla campos genéricos y todas las entidades relacionadas.

## Goals / Non-Goals

Ver proposal.md. Reutilizar ADR0049, PageTools y operaciones existentes; no añadir jerarquía, vistas Recent/Fav/A-Z/By Tag/Voice con reglas de dominio nuevas ni documentos de Tag. Las secciones usan List y datos canónicos disponibles.

## Decisions

1. **Propiedades.** TagPageProperties recibe payload y HomeSummary/tag. Type ofrece las cuatro clasificaciones nativas y conserva desconocidos; Favorite prepara el draft. Note Count usa home.query/tags_alphabetical con limit100 existente. Fuera de esa página o en archivo de origen, resumen no disponible; error permite reintentar. El contador desplaza/enfoca Notes. Actividad/procedencia siguen en EntityFields secundario de lectura. No se deriva un conteo en el cliente ni se copia el de Notion.
2. **Contenido.** TagContents recibe workflow/context actual con cursor común. Presenta Notes normales y Clips por note_type web_clip; Goal/Milestone y entidades de contexto permanecen en cabecera. Tasks conserva un grupo nativo desplegable con acceso y captura, incluso cuando está vacío. Crear Clip reutiliza createHome con tipo y ContextLink, no simula una captura web. Paginar acumula IDs únicos; ausencia parcial explícita y errores independientes. Sin nuevas reglas de orden/filtro en Svelte.
3. **Composición y conservación.** Portada decorativa reutiliza el patrón Project sin descargar su imagen. Icono original administrado, Nav real a Notes/Tags y relaciones originales. PageTools queda intacto: sólo Recursos externos en Tag. Generaciones/workspace/ID protegen lecturas tardías, igual que Projects. UI/CLI/MCP mantienen reglas Rust y guardado CAS existentes.

## Risks / Trade-offs

- Resumen fuera de los primeros100 tags → No disponible, nunca cero; no ampliar cuotas para un detalle.
- Contexto parcial y reglas de archivo diferentes de Home → expresar página cargada; contador canónico separado de las filas del contexto previo, sin ocultar datos silenciosamente.
- Herencia/relaciones de Notion ausentes localmente → no copiar controles ni referencias inexistentes.
- Revisión de borrador sobre copia → no acredita persistencia, jerarquía, proveedor o IPC nativo.

## Migration Plan

Sin cambios persistentes, Rust, contratos, dependencias o migración. Baseline/backup canónico previos fuera de Git. Revertir UI conserva datos. Meeting Notes sigue0/19 y no se operan Eco Armario, voicebot ni la pantalla nativa.
