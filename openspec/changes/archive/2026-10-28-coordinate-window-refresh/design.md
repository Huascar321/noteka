# Design

## Context

Cada `home.query` tiene snapshot propio. `ReadQueue` ya conserva el cupo físico hasta terminar IPC, pero sus instancias son locales y una nueva generación descarta la anterior. La UI usa `WorkspacePane` y lectores independientes por sección; las ediciones se guardan con revisión esperada.

## Goals / Non-Goals

**Goals:** acotar el trabajo duplicado, conservar contexto visual e informar datos no verificados sin perder ediciones.

**Non-Goals:** polling, `data_version`, snapshot común entre secciones, migración SQL, cambiar el guardado explícito o relajar los lectores especializados.

## Decisions

- Un pool por ruta canónica de workspace limita a dos las operaciones de lectura usadas por refrescos; también abarca esas lecturas iniciadas por interacción. Escrituras no esperan este pool. El pool se retira sólo al terminar físicamente el último IPC y vaciar su cola.
- Cada lector Home agrupa peticiones de la misma vista y fecha: antes del inicio físico se absorben en la primera consulta; durante una consulta dejan una única pasada posterior. Un cambio de identidad cancela encoladas y descarta respuestas antiguas.
- Los paneles de listas delegan directamente al lector, y las demás rutas agrupan eventos por panel para evitar ciclos anidados. Un evento durante guardado o bloqueo queda pendiente hasta que el panel pueda leer.
- Un estado interno distingue ocupado, resultados anteriores y reinicio de páginas. La consulta sin cursor preserva la fecha explícita; una selección de vista/fecha nueva invalida identidad. Los controles de paginación y movimiento no usan datos anteriores.
- Las relaciones del detalle se resuelven de forma acotada y se invalidan al navegar. Las respuestas de entidades mantienen las comprobaciones de revisión y el buffer de edición.
- El benchmark compara la misma UI y CLI sintético antes/después; el foco físico se verifica por separado con una ventana Tauri temporal autorizada.

## Risks / Trade-offs

- [Más de una página] → se vuelve a la primera incluso sin cambios; aviso explícito en la interfaz.
- [Escrituras externas posteriores a una lectura] → se observan al siguiente foco/actualización; no se promete una instantánea global ni sincronización continua.
- [IPC en curso] → sólo se descarta su resultado; el cupo no se libera hasta su terminación real.

## Migration Plan

No existe migración persistente; reiniciar el proceso de escritorio para cargar el frontend nuevo. Las preferencias, originales y borradores conservan sus contratos.
