# Design

## Context

SQLite y originales administrados son la fuente de verdad. El workspace compartido usa WAL y `Workspace::open` conserva prevalidación e integridad. Desktop, CLI y MCP dependen del mismo core y deben compilar la misma biblioteca incluida.

## Goals / Non-Goals

**Goals:** actualizar el motor incluido, comprobar la versión efectiva y medir antes/después con corpus sintéticos repetibles.

**Non-Goals:** alterar el esquema, operaciones, migraciones o refrescos de interfaz; modificar bases personales; inferir que un control de integridad prueba ausencia de todo daño.

## Decisions

- Usar rusqlite 0.41.0 con `bundled`, `backup` y `hooks`; resolver libsqlite3-sys 0.39.1 en Cargo.lock. Su SQLite 3.54.1 contiene la corrección WAL, a diferencia del motor anterior.
- Compilar los tres clientes sobre la misma dependencia y verificar `sqlite_version()` en Rust y los binarios enlazados. Los procesos viejos deben reemplazarse para aprovechar la corrección.
- rusqlite 0.41 exige acceso mutable para instalar el hook de progreso, incompatible con transacciones que sólo exponen `&Connection`. Un guard privado usa la API C con memoria estable y un hook compartido entre guards anidados, que aplica todos los deadlines activos y desregistra antes de liberar su contexto. La conexión se mantiene prestada y no es `Sync`; la interrupción, anidamiento y limpieza ante panic se prueban explícitamente. No se desactivan los plazos existentes.
- Comparar muestras en release sobre corpus temporales de 1.000 y 8.000 entidades, 30 ciclos de seis vistas por corpus, con cantidad de operaciones, tiempos y memoria residente máxima. Son medidas de backend, no latencia completa de ventana.
- Ejecutar las pruebas existentes de workspace, migraciones, backup/restauración, documentos desconocidos y clientes sobre fixtures temporales; mantener errores y límites sin relajarlos.

## Risks / Trade-offs

- [Dependencia Rust con cambios de API] → compilar los consumidores y ajustar sólo incompatibilidades necesarias.
- [Clientes anteriores todavía en ejecución] → documentar que deben reiniciarse todos antes de usar el motor corregido.
- [Pruebas ya fallidas] → separar línea de base y regresiones; conservar el resultado original y no alterar expectativas para ocultar defectos.

## Migration Plan

No se modifica el formato persistente ni se ejecuta una migración nueva. La actualización de los binarios se valida antes de continuar; la reversión del binario conserva el mismo esquema, pero volvería a exponer el error de WAL.
