# ADR 0008: instancias de plantilla y referencias a vistas

12 de octubre de 2026. Adoptado e implementado en `add-collection-templates-and-embeds`; [verificación](../verification-phase-3c-templates.md).

## Context

El motor tiene una identidad compartida por entidad/registro y cinco presentaciones con reglas Rust. Plantillas e incrustaciones no deben crear otro motor ni convertir resultados de consulta en texto original.

## Decision

- Plantilla independiente, definición JSON v1 cruda recuperable y revisión propia; defaults estáticos por ID de propiedad, sólo páginas note. `template.save/get/list/instantiate` comparten contrato entre desktop y CLI.
- Instanciar compara revisiones de plantilla/colección bajo una transacción Immediate. Un helper de creación recibe esa transacción y confirma entidad/documento/membresía/valores/historial/FTS juntos; entity.create y record.create conservan su API.
- Nuevos IDs para entidad y todos sus bloques, pero IDs explícitos de adjuntos/relaciones/vistas conservados. Omisión conserva default; null limpia. Validar defaults y overrides completos antes de fusionar.
- Documento v3 con hoja `collection_view` de dos IDs; SQLite v7 con tabla templates y backup previo, sin reescribir JSON previo. Manifiestos de backup mantienen sus formatos.
- La referencia se valida estructuralmente y por existencia/pertenencia al escribir; la ejecución de la vista se valida al leer. Un destino futuro no obliga a convertir el documento.
- Presentación pura compartida y lector incrustado de lectura paginada, sin expansión de documentos, previews ni snapshots persistidos. Apertura usa las guardias del editor; actualización no escribe la nota.
- Export individual conserva referencias y advierte que no incluye destinos transitivos. FTS sólo indexa fuentes; backup transporta el workspace completo.

## Alternatives

Notas ocultas para templates mezclarían Inbox/archivo/búsqueda; clone-by-ID haría defaults dependientes de cambios de la página fuente. Crear entidad y luego aplicar defaults en otra transacción permite estados parciales. HTML/iframe/snapshots incrustados acoplan el modelo al editor, red o datos derivados. Duplicar DatabasePanel expone escritores y buffers anidados; se comparte sólo la presentación.

## Consequences

Clientes hasta v6 rechazan v7; rollback por restauración del backup previo a **otra** ruta, no downgrade con pérdida. UI inicial de templates usa JSON explícito; no automatizaciones ni templates de esquemas/built-ins. El Home, Goals, fotos y clima siguen fuera del cambio. Paginación no ofrece snapshot global entre llamadas.
