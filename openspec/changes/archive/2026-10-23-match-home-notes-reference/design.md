# Design

## Context

HomeReader ya aísla generaciones, errores y cursores. NotesAll conserva todas las páginas importadas como notas; el importador calculó procedencia pero no la persistió. El export y manifiesto locales permiten distinguir 195 filas de Notes sin clasificar por nombre.

## Goals / Non-Goals

**Goals:** fidelidad visual y vistas sustentadas por datos originales, conservación y recuperación reversible.

**Non-Goals:** grabar/transcribir voz, automatizaciones, servicios remotos, reinterpretar cuerpos o migrar bloques.

## Decisions

1. Campos opcionales `note_type`, `favorite`, `source` en NoteFields. Source contiene proveedor/colección, record_id, created_at/updated_at y archived de origen. Validación Rust, sin tabla ni migración. Notes sin source siguen siendo notas locales; Source de otra colección se conserva en All. No inferir colección por título. Alternativa de filtrar por listas de IDs en UI descartada porque rompería paginación y clientes.
2. NotesLibrary incluye notas locales y Notes de fuente no archivadas, excluyendo web_clip/voice/journal/meeting. Categorías y favoritos usan valores explícitos; Inbox conserva su marca. All conserva semántica previa. Las vistas nuevas ordenan por modificación original cuando existe y local en otro caso, con ID de desempate. Los días de fuente se etiquetan como tales; no se reescribe updated_at local para simular antigüedad.
3. Extender proyección HomeMetadata, manteniendo límites y snapshots de Home. Tipo/favorito editables en detalle; procedencia sólo visible. Un selector crea con tipo/favorito explícito en la vista elegida, sin writes hasta guardar.
4. Recuperación desde manifiesto y CSV por identidad inequívoca. Source se anota en las páginas importadas; tipos/favoritos/timestamps sólo si se acreditan. Relaciones e iconos visibles se vinculan por ID original y coincidencias únicas. Backup canónico antes de aplicar; informe incremental y CAS por entidad. Duplicados se resuelven sólo con Created/Updated y propiedades del bloque inicial del Markdown, sin su cuerpo. Si el export tiene fechas antiguas se contrasta metadata actual de Notion por ID original, sin pedir contenido. El contenido del documento nunca se envía como update ni se inspeccionan cuerpos de notas de la referencia.
5. HomeNoteSection comparte lenguaje visual de Tasks. Iconos nativos Lucide, imágenes personalizadas mediante image-UUID ya soportado. Filas sin bordes, metadatos a la derecha, footer discreto y reglas desplegables; transiciones existentes respetan movimiento reducido.

## Risks / Trade-offs

- [Export sin zona explícita] → conversión de timestamps con zona del workspace declarada en el informe; no sobrescribir fechas locales. La antigüedad refleja la fuente exportada.
- [Payload nuevo frente a cliente anterior] → backup y actualización del cliente antes de editar; lectura estructurada conserva campos.
- [Enriquecimiento y buffers abiertos] → revisiones normales, nunca forzar un guardado sobre otra revisión.
- [Relaciones incompletas del export] → recuperar únicamente las visibles y acreditadas; declarar alcance y no inventar las restantes.

## Migration Plan

Compilar núcleo/contratos/UI, generar preview de enriquecimiento, respaldar, aplicar operaciones con CAS, revisar resultados y documentos intactos. Reversión por informes y revisión vigente; backup completo sólo con elección explícita para no perder cambios posteriores. No ejecutar suites no solicitadas.
