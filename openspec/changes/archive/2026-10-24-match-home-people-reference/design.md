# Design

## Context

PersonFields sólo conserva icono/color. Home usa una galería textual de 17 personas: 11 contactos del CSV People y seis miembros de otra colección homónima. El Home de Notion acreditado tiene siete vistas; Contacts es galería pequeña con page_cover, relaciones y empresa. Ver proposal.md y deltas.

## Goals / Non-Goals

**Goals:** representar los contactos reales y sus fotos originales offline, reglas compartidas y siete vistas utilizables, con recuperación reversible.

**Non-Goals:** sincronización remota, Google Contacts/Fotos, mensajes, recordatorios, CRM completo o arrastre de Pipeline. Se edita pipeline desde el detalle existente. No se ejecutan suites.

## Decisions

1. PersonFields añade campos opcionales. Relaciones son etiquetas de fuente, hasta 32 etiquetas únicas de 128 bytes; empresa hasta 512 bytes; foto UUID local y posición vertical entera 0..100; fechas canónicas; cinco estados de pipeline; procedencia NoteSource reutilizada y gallery_order 0..10000. Se evita migración SQL y se mantienen payloads antiguos. Los clientes con validadores cerrados deben actualizarse antes de editar los campos nuevos.
2. HomeMetadata devuelve PersonFields interpretados y ocurrencia de cumpleaños del mes consultado. Rust usa procedencia People/no procedencia para la biblioteca; WorkspacePeople permanece fuera de Home y accesible en el listado general. New/Updated usan fecha de fuente con fallback local y límite inferior inclusivo de un mes civil; Check-Ins fecha <= un mes civil futuro, incluyendo atrasados. Contacts/New/Updated/Pipeline conservan orden acreditado de Contacts, luego título/ID; Check-Ins ordena fecha. No se replica una fórmula remota no disponible.
3. Birthdays usa as_of existente para mes civil navegable y calcula ocurrencia anual en Rust. El 29/2 se muestra el 28/2 en años no bisiestos; la fecha original no cambia. Sin edad inferida ni LLM. New/Updated sin resultados actuales muestran vacío, no actividad de la recuperación.
4. Active Projects consulta proyectos Doing/Ongoing no archivados en origen y relaciones Project→Person válidas antes de paginar; personas activas y admisibles. Agrupa cada proyecto por cada contacto y conserva su resumen de tareas. Pipeline incluye cinco columnas y grupo sin estado sólo si corresponde. Límites actuales de candidatos, relaciones, fuentes y grupos permanecen.
5. Componente People específico y fotos con lector/cache acotados existentes, descarte tardío por workspace/ID y fallback neutro. Contacts carga diez registros y acumula páginas; las demás vistas usan hasta cien para que grupos/calendario no fragmenten la pequeña referencia. Los grupos de páginas acumuladas deben combinarse por clave e IDs, no sólo reemplazarse.
6. Recuperación desde el manifiesto, CSVs y propiedades iniciales, sin cuerpos. Metadata actual autorizada de Contacts, orden/recorte y nueve fotos visibles se guardan fuera de Git. Se resuelven proyectos por el ID de procedencia ya acreditado; los inexistentes quedan en informe. Backup antes de importar adjuntos o actualizar; CAS y registro incremental por escritura, sin documento en el update. Se comparan todos los IDs y digest documental antes/después.

## Risks / Trade-offs

- Export desactualizado → fechas actuales de metadata autorizada con procedencia; no se cambian fechas locales ni se promete sincronización.
- Orden de fórmula opaco → orden acreditado y fallback explícito; no se inventa apellido por partición del nombre.
- Foto ausente → espacio neutro; no se generan caras ni se usan asociaciones por título.
- Recuperación parcialmente completada ante fallo → informe incremental con payload/revisión anteriores y backup canónico; revertir por revisión tras inspección, nunca restaurar ciegamente sobre ediciones posteriores.
- Aceptación headless → acredita compilación/proyección/composición, no transporte o cierre Tauri nuevos.

## Migration Plan

Sin DDL. Copiar baseline externo, implementar campos/proyecciones, compilar CLI/contratos/UI, dry-run y backup/recuperación explícita. Inspeccionar manualmente copia aislada en temas claro/oscuro y ancho estrecho; verificar documentos/IDs. Actualizar guía/ADR/verificación, sincronizar sólo requisitos completados y archivar.
