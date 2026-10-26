# Design

## Context

La referencia usa título de32px, filas de32px, grupos y columnas alineadas. La UI anterior usaba encabezado19px, subrayados/separadores y metadatos de depuración visibles. Home dispone de proyecciones Rust acotadas y contextos reales; el export previo omitió iconos de Notion.

## Goals / Non-Goals

**Goals:** fidelidad de Tasks y arte local auténtico, con controles útiles, preservación de los datos y offline.

**Non-Goals:** paridad de Recurring/Active Content, automatizaciones/IA, copiar notas o credenciales, migrar SQLite o recalcular reglas de negocio en Svelte.

## Decisions

1. Componente Tasks especializado dentro de Home; listas del resto no cambian. Creación y apertura usan callbacks/guardias previos. Reglas/cobertura se conservan en un control de detalle y footer compacto. Sólo se muestran herramientas reales; sin botones ficticios de automatización.
2. Vistas Rust adicionales. Week compara semana ISO de due_date; Month año/mes civil; Scheduled exige due_date y No Due su ausencia. Todas exigen estado todo. Proyectos activos admite planned/doing/ongoing y en progreso doing/ongoing; excluye archivados. Se filtra por relaciones antes de paginar y sólo se agrupa por proyectos elegibles. Las vistas nuevas de proyectos ordenan tareas por creación local descendente; no se acredita el orden histórico de Notion. Criterio nuevo implica identidad de cursor distinta. Today no se convierte en My Day.
3. Clave `image-<uuid sin guiones>` en payload.icon, dentro del límite existente40bytes. UUID refiere attachment existente. No cambia DDL ni el shape del payload. Las claves Lucide antiguas que comienzan image- siguen siendo válidas; la nueva interpretación sólo reconoce el patrón completo image-UUID. Clientes anteriores lo conservan como clave desconocida y usan fallback. Rust conserva la validación de claves anterior; el renderer acepta sólo UUID hexadecimales completos para image-. La lectura confirma integridad/MIME del adjunto y nunca interpreta SVG/HTML. La elección de color no reemplaza una imagen por una clave Lucide implícita.
4. AppIcon comparte un resolvedor de imágenes con contexto de workspace, cola de2 y cache acotada a32 imágenes/1MiB cada una. Cambiar fuente/workspace o desmontar invalida publicación; las llamadas físicas conservan su cupo hasta terminar. Imágenes ausentes fallan a Lucide sin bloquear datos. La imagen se obtiene localmente, sin URL firmada ni ruta arbitraria.
5. Recuperación explícita de recursos observados en Notion; stock de interfaz se empaqueta localmente y logos personalizados quedan como adjuntos del usuario. Se conserva backup y un manifiesto externo de correspondencia/revisiones. Coincidencias ambiguas o entidades con icono propio se omiten; no se cambian documentos, fechas o estados para igualar la captura. Se añaden sólo relaciones tarea/proyecto presentes en la referencia, emparejando título y fecha con entidades locales inequívocas; un duplicado queda pendiente hasta acreditar identidad con el ID original, propiedades exactas y trazabilidad completa al manifiesto; se conservan vínculos existentes. Sin capturas privadas/URLs/datos personales en Git.

## Risks / Trade-offs

- El formato icon admite claves desconocidas: image- queda documentado y validado; no se amplía a URLs ni HTML.
- Lectura de adjunto puede fallar o ser lenta: fallback inmediato, cupo compartido, cache acotada y publicación vigente.
- Tareas en varios proyectos aparecen por grupo; conteos son de página, no un total global de filas.
- La referencia contiene vistas sin equivalente: quedan como pendientes explícitos, sin simular resultados.
- El usuario no pidió ejecutar suites de tests en esta corrección; se usan compilación/lint documental e inspección visual y se explicita ese alcance.

## Migration Plan

Sin cambio SQLite/API de operaciones. Se amplía enum HomeView y tipos generados; se preservan variantes previas. Antes de enriquecer iconos del workspace se usa backup canónico. Revertir la presentación restaura sólo este diff; las referencias de imagen/adjuntos originales se conservan, con fallback en clientes anteriores.
