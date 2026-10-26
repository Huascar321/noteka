# ADR 0042 — Metadatos de personas y galería local

Fecha: 24/10/2026. Estado: aceptada para el alcance de People.

## Context

People necesita distinguir contactos de miembros importados de una colección
homónima y presentar propiedades/fotos reales. El núcleo comparte reglas entre
escritorio, CLI y MCP; SQLite y adjuntos originales siguen siendo autoridad.

## Decision

Extender PersonFields con propiedades opcionales, procedencia NoteSource existente
y orden de galería acreditado. Validar etiquetas, empresa, UUID, posición, fechas,
pipeline y orden en Rust. Sin nuevas tablas ni operaciones. Leer propiedades
interpretables y filtrar antes de paginar; metadata inválida informa cobertura
no disponible, sin reparación automática.

Guardar fotos originales como adjuntos locales y usar el lector/cache de imágenes
ya acotado. Asociarlas por ID original observado, nunca por parecido o nombre.
El recorte vertical se expresa como porcentaje entero. No se guardan imágenes
personales ni referencias remotas de ejecución en Git.

Usar fechas civiles del workspace para períodos y calendario. El 29/2 ocurre el
28/2 en años no bisiestos, preservando la fecha original. Recuperar metadatos no
cambia fechas locales de creación ni fabrica actividad reciente.

Consultar Project→Person antes de paginar Active Projects, validando las personas
admisibles. Las filas muestran tags/Goals y progreso local; cada grupo conserva
IDs de originales. Projects Board sigue presentando sólo tags/Goals en sus
tarjetas, para que los vínculos de persona recuperados no alteren su composición.

## Alternatives

- Fotos mediante URLs de Notion: descartado porque dependería de red y sesión.
- Contactos como notas o colección duplicada: descartado porque fragmentaría IDs
  y la validación del dominio existente.
- Clasificar por título o deducir apellidos: descartado por ambigüedad; se usa
  identidad original y orden observado, con fallback de título/ID para locales.
- Filtros en Svelte: descartado porque cambiarían resultados al paginar y romperían
  consistencia con CLI/MCP.

## Consequences

Los payloads anteriores siguen válidos. Los clientes con esquemas cerrados deben
actualizarse para editar payloads enriquecidos. La recuperación requiere backup,
CAS e informe incremental privado; sólo añade campos ausentes y relaciones
acreditadas, sin cuerpos. No es sincronización continua ni aceptación nativa nueva.
