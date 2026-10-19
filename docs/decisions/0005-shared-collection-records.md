# ADR 0005: registros como membresías de entidades

- Fecha: 12 de octubre de 2026.
- Estado: adoptada en el motor de colecciones inicial.

## Decisión

SQLite v4 almacena colección/esquema/membresía/valores/vistas. Registro no añade otro ID de página: referencia una entidad existente. Notas nuevas de colección tienen bloques y `inbox:false`; built-ins existentes mantienen tipo/payload/relaciones. Título y timestamps se proyectan desde entidad, no se duplican como autoridad.

Valores son patches por ID de propiedad. Membresía y valores se confirman con escritura de entidad/historial/índice; creación usa respuesta transaccional antes del commit. Esquema/valores/vistas emplean sus revisiones explícitas. Referencias de archivos verifican originales y las relaciones genéricas consultan inversa sin persistir una segunda copia. Tipos/opciones no se cambian automáticamente.

Decimales monetarios son strings con moneda fija del esquema. Comparación exacta usa mantisa i128/escala nueve; se conserva el string original. No usar floats para dinero, redondeo ni cálculos hasta el cambio de fórmulas.

## Alternativas y consecuencias

- Tablas de páginas nuevas independientes: duplicarían identidad y navegación con notas/tareas; descartadas.
- Convertir todos los built-ins de inmediato a esquema genérico: añade migración de payload sin necesidad para 3a; se mantiene su autoridad actual y se proyecta progresivamente.
- Valores en columnas físicas por propiedad: exige migración por cada campo; se elige JSON tipado con FK de propiedad/entidad.
- Inversa mutable duplicada: puede divergir; se consulta desde el valor único.
- Consulta SQL dinámica completa o TanStack como dominio: innecesaria para las reglas acotadas iniciales. Rust valida/evalúa, tabla/lista nativas presentan páginas; revisar con corpus medido antes de ampliar.

Los formularios JSON/IDs son una superficie funcional deliberada, no la UX final de bases. Colecciones hasta 10.000 miembros y 128 propiedades; su costo in-memory está limitado pero todavía no medido. El núcleo no interpreta código ni formatos futuros al escribir. [Contrato y límites](../collections.md).
