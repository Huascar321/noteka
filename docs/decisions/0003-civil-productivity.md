# ADR 0003: productividad civil y captura contextual

- Fecha: 12 de octubre de 2026.
- Estado: adoptada para fase 2.

## Decisión

Las fechas de planificación/vencimiento son fechas civiles canónicas YYYY-MM-DD, no medianoches UTC. El día por defecto lo calcula Rust usando la zona IANA del workspace, y cada consulta devuelve el día usado. Instantes de creación/modificación siguen UTC. Inbox, My Day, vencidas y pendientes son filtros explícitos documentados en [productivity.md](../productivity.md), no prompts ni inferencias.

Los metadatos se agregan al payload de la misma entidad, sin tablas duplicadas para PARA. SQLite permanece v2: lectura no reescribe originales antiguos. `entity.update.payload` es un patch validado dentro del lock de revisión; omitir una fecha la conserva, null la limpia. Completar con un cambio de estado no elimina planificación ni prioridad. Se valida la fusión completa, por lo que un campo futuro no puede desaparecer al sustituirlo por un payload menor. Título/archivo pueden cambiar sin reemplazar un payload desconocido.

Captura contextual es una sola operación `entity.create` con contexto opcional. Entidad, documento, relación e historial se confirman juntos o no se confirma ninguno. El dashboard consulta ambos extremos con EXISTS y abre los mismos IDs; no hace copias ni sincroniza dos relaciones inversas.

## Alternativas y consecuencias

- Instantes para fechas sin hora: resultados dependientes de zona y desplazamientos inesperados; descartado.
- Inbox calculado por falta de tags/proyecto: clasificar podría sacar contenido involuntariamente; bandera explícita elegida, defaults compatibles para datos anteriores.
- My Day automático por vencimiento: mezclaría selección del día y atrasos; se separa Overdue.
- Crear y relacionar en dos requests: captura parcial ante fallo; descartado.
- Reemplazar payload completo: facilita borrar campos omitidos o desconocidos; patch estricto conservador.

Los tipos built-in siguen entidades estables. Colecciones genéricas se agregarán conservando identidad en 3a; no se anticipa un segundo sistema de registros.
