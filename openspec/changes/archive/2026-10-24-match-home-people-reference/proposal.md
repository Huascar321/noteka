# Proposal

## Why

People todavía muestra tarjetas textuales grandes y mezcla contactos con miembros del workspace. La referencia autorizada usa una galería compacta con fotos, relaciones y empresa, y siete vistas con reglas distintas.

## What Changes

- Galería inicial con portadas locales originales, chips de relación, empresa, New y carga incremental; encabezado y controles coherentes con Tasks/Notes/Projects/Tags.
- Metadatos opcionales de persona validados en Rust: relaciones, empresa, foto/posición, cumpleaños, check-ins, pipeline, procedencia y orden acreditado.
- Contacts/New/Updated/Check-Ins/Birthdays/Pipeline/Active Projects reales, con filtros previos a paginación, calendario mensual y proyectos agrupados por persona.
- Recuperación acotada del export/manifiesto y metadata autorizada de Notion, con backup, CAS, informe privado y documentos/IDs conservados. Los miembros importados siguen en el listado general.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: composición People, vistas compartidas y recuperación explícita.
- `knowledge-entities`: propiedades opcionales de personas y validación conservadora.

## Impact

Rust contracts/productivity/home, contratos generados, HomePanel y componentes People, campos del detalle, estilos y helper de recuperación. Sin migración SQL, nuevas dependencias, sincronización continua, mensajes, notificaciones ni envío a proveedores. Pipeline es una proyección editable desde el detalle; su arrastre queda fuera del alcance. Verificación por compilación e inspección manual sobre una copia aislada; no se agregan ni ejecutan suites.
