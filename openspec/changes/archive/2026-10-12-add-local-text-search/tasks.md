# Tasks

## 1. Índice y contrato
- [x] 1.1 Migrar SQLite v3 e indexar/reconstruir transaccionalmente; probar originales, rollback, revisiones, archivo, migración/backup y formatos desconocidos; documentar diseño.
- [x] 1.2 Añadir consultas, filtros/provenance/paginación y CLI; probar literales/invalid-input y paridad, regenerar contrato y documentar comandos.

## 2. Escritorio e integración
- [x] 2.1 Integrar búsqueda con apertura del original/guardia y localizador, probar nativamente sin red; medir consultas release en 10.000 notas y registrar método/resultados.
- [x] 2.2 Ejecutar regresiones Rust/adaptador/nativas, build release, fmt/Clippy/OpenSpec y revisar diff/escenarios/diseño con evidencia.

## Workflow follow-up
- Sincronizar/archivar lo verificado. El siguiente tramo de 5a deberá implementar cola/extracción; no declarar esa fase completa por entregar sólo FTS.
