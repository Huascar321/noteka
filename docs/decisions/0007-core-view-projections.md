# ADR 0007: filtros acotados y proyecciones de vistas compartidas

- Estado: adoptada en `add-advanced-collection-views`.
- Fecha: 12 de octubre de 2026.

## Contexto

3a/3b ya conservan entidades y valores originales, pero tabla/lista y AND simple no cubren todas las presentaciones previstas. El Home confirmado por capturas necesita esas bases, sin que una pantalla se vuelva una segunda implementación del dominio.

## Decisión

Rust valida un árbol pequeño all/any/not/condition y comparadores tipados, reutilizando igualdad/decimal/errores existentes. Todas las referencias se validan antes de seleccionar resultados; un error calculado referenciado no se oculta por cortocircuito. Los filtros legacy continúan siendo AND.

Database query devuelve registros planos únicos y proyecciones por ID de su página: tablero select/checkbox, calendario mensual civil date/date_range y galería textual sin lectura automática de originales. Grupos/días vacíos se conservan, matching_count registra cobertura total antes de paginar. El cliente renderiza y publica config/layout/datos conjuntamente; un fallo retira resultados viejos.

SQLite v6 señala la evolución del formato de configuración, aunque no requiera otra tabla. Migración 006 conserva JSON de vistas y crea backup previo; cliente viejo rechaza versión nueva. Formato futuro desconocido no se reemplaza mediante deserialización parcial. [Contrato](../views.md).

## Alternativas

- Agrupar/filtrar en Svelte: duplicaría reglas y divergencias CLI/UI; descartado.
- DSL SQL o filtros sin límites: ampliarían superficie y costo; descartado.
- Cargar toda colección en UI: sustituido por proyecciones de la página y cobertura explícita.
- Cambio de formato sin subir SQLite: permitiría a clientes viejos abrir datos nuevos antes de fallar; descartado.
- Descarga/decodificación automática de covers: requiere presupuesto de memoria/política aparte; galería textual inicial, no paridad de fotos People.

## Consecuencias

Los grupos parciales no deben anunciarse como totales. Rangos repiten referencias, nunca entidades. Null/NOT no imitan lógica SQL; reglas están documentadas. No hay snapshot multipágina ni benchmark de vistas. Formularios JSON/IDs, sin drag-and-drop, son provisionales. Esta entrega no completa templates/bases incrustadas ni copia completa del Home.
