# ADR0039 — Tipos de tags y actividad acreditada

Fecha: 23/10/2026. Estado: aceptada.

## Contexto

La referencia agrupa tags en Area/Resource/Entity, mientras el núcleo conserva
Topic. Los contadores estaban incompletos y las actualizaciones de recuperación
alteraban timestamps locales que no equivalen a modificación de la nota fuente.

## Decisión

Entity se añade como clasificación independiente; Topic conserva su significado.
TagFields añade favorite, source y source_activity_at opcionales. Rust valida
estos metadatos y los filtros generales, sin cambiar el esquema SQLite.

El contador y la última nota se derivan de relaciones locales únicas y activas,
excluyendo archivo de origen. Se prefiere modificación de fuente acreditada para
notas importadas. Recent combina el snapshot explícito de actividad con fechas
de notas relacionadas antes de paginar, en la misma transacción y bajo los caps
existentes. No se presenta como fórmula universal Latest Activity de Notion.

La recuperación usa identidades del manifiesto y bloques iniciales de propiedades,
con backup, CAS e informe incremental privado. Los documentos no se envían como
updates. La corrección del índice de títulos completos repara dos procedencias
heredadas equivocadas; no se agregan reglas por título para seleccionar iconos.

## Alternativas y consecuencias

Renombrar Topic como Entity alteraría datos existentes y se descarta. Copiar
contadores/última nota de Notion produciría datos obsoletos al editar localmente y
también se descarta. Las fechas importadas siguen siendo snapshots y los clientes
cerrados anteriores necesitan actualizarse antes de editar estos payloads.
