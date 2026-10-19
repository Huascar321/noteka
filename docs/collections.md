# Colecciones locales: motor inicial (3a)

Una colección contiene membresías de entidades compartidas. `record.create` crea una nota con bloques, fuera de Inbox; `record.attach` incorpora una entidad existente sin copiarla. Una tarea sigue siendo una tarea, con su payload y relaciones. Abrir un registro abre el editor original; editar título/contenido se hace por `entity.update`.

## Uso en escritorio

Collections → Nueva colección → Definir propiedad → Nuevo registro. Las presentaciones muestran valores y permiten editar un patch por ID de propiedad. El esquema muestra los IDs. Incorporar entidad por ID exige su revisión vigente; el núcleo comprueba pertenencia y tipos. Se pueden guardar y editar vistas reutilizables con layout, columnas, filtros y orden. Tabla/lista fueron el alcance inicial de 3a; [vistas avanzadas](views.md) agrega tablero/calendario/galería y árboles de filtros.

**UX inicial:** definición de propiedades, valores y configuración de vistas usan formularios JSON explícitos. No son pickers ni fórmulas ejecutables, y no se promete paridad visual con Notion. Un buffer protege navegación/cierre; cancelar conserva, descartar no escribe, guardar y continuar confirma primero. Conflictos no actualizan automáticamente su revisión. Recuperar buffer JSON permite copiar el texto pendiente, incluso JSON inválido.

## Operaciones compartidas

Entrada/salida completa: `operations describe --json`; DTOs regenerados en `src/lib/contracts.generated.ts`.

| Grupo CLI / operación | Entrada principal | Salida |
| --- | --- | --- |
| `collection create` | `{name}` | colección + propiedades + vistas |
| `collection get` / `list` | `{id}` / `{limit?,cursor?}` | bundle / página de colecciones |
| `property define` | `{collection_id,expected_revision,spec}` | propiedad con ID estable |
| `record create` | `{collection_id,title,values?,document?}` | entidad + valores + membresías |
| `record attach` / `update` | `{collection_id,entity_id,expected_revision,values?}` | registro confirmado |
| `record get` | `{collection_id,entity_id}` | registro original + proyecciones |
| `record inverse` | `{property_id,target_id,limit?,cursor?}` | página de registros fuente |
| `view save` | `{collection_id,name,layout,config,id?,expected_revision?}` | vista guardada |
| `database query` | `{collection_id,view_id?,config?,layout?,limit?,cursor?}` | propiedades/config/layout + página, matching_count y grupos por ID |

`view_id` excluye `config`/`layout`. Vistas nuevas no llevan revisión; actualizar exige ID/revisión. `entity.create` acepta `record:{collection_id,values?}` para confirmar captura built-in y pertenencia en una transacción. `record.create` reutiliza esa transacción y arma la respuesta **antes** del commit, evitando comunicar fallo tras una lectura posterior a una creación ya confirmada.

```json
{"name":"Cuota","type":"money","currency":"BOB"}
```

```json
{"columns":["id-titulo","id-cuota"],"filters":[{"property_id":"id-cuota","op":"eq","value":"48.570"}],"sorts":[{"property_id":"id-cuota","descending":false}],"archived":false}
```

Filtros legacy hasta 16, combinados AND: `eq` tipado, `contains` textual sensible a mayúsculas, `is_empty` sin value. 3c agrega ne/lt/le/gt/ge y [árboles AND/OR/NOT acotados](views.md). Hasta cuatro órdenes, nulos al final incluso descendente, desempate por ID; texto en orden Unicode/binario, arrays/objetos en representación JSON determinista. No admite SQL ni expresiones cliente. `"48.570"` y `"48.57"` comparan iguales numéricamente sin alterar el texto original.

## Tipos, vacíos y precisión

Toda colección tiene `title` proyectado; no se define otro. `created_at`/`updated_at` son instantes proyectados. Se edita título por operaciones de entidad, no por valores. Los timestamps conservan UTC y el escritorio usa la zona del workspace.

- `text`: string hasta 16 KiB; string vacía es distinta de null.
- `number`: número JSON finito de magnitud ≤2^53−1, no dinero. Clientes JS no deben enviar enteros mayores.
- `decimal`/`money`: **string** sin exponente, máximo 28 dígitos y 9 decimales, sin ceros iniciales ni cero negativo. Mantisa i128 con escala común para comparar; nunca float ni redondeo implícito. `money` exige moneda fija ASCII de tres mayúsculas en el esquema; no verifica un registro externo de monedas.
- `checkbox`: booleano; false no equivale a vacío.
- `date`: fecha civil YYYY-MM-DD validada; `date_range`: `{start,end}` civiles, start ≤ end.
- `select`/`multi_select`: 1–100 opciones únicas declaradas; multiselección array sin duplicados. Orden del array se conserva.
- `url`: http/https, sin espacios/controles; `email`: sintaxis conservadora, sin verificar DNS, acceso ni entregabilidad. Valores no se abren automáticamente.
- `files`: array de hasta 100 IDs de originales importados, sin duplicados; al escribir se verifica hash/archivo administrado. Backup incluye originales usados como propiedad, aunque no estén en bloques. Quitar referencias no elimina archivos.
- `relation`: array de hasta 100 IDs miembros de `target_collection_id`, sin duplicados. La proyección inversa se consulta desde ese mismo valor; destino archivado no se elimina.

`values` es un patch de IDs: omitir conserva, null elimina el valor, array/string vacíos se conservan. Tipos/opciones/destinos de propiedades son inmutables en 3a. Cambios de tipo o eliminación física necesitarán migración recuperable, no coerción silenciosa. Propiedades ajenas, campos futuros y tipos incompatibles se rechazan. Un valor futuro puede leerse como JSON, pero no se reemplaza ni limpia mediante una escritura que no lo reconoce; consultas tipadas fallan explícitamente antes de tratarlo como un dato válido.

## Persistencia y límites

SQLite v4 agrega collections/properties/memberships/property_values/views; migración 004 crea backup previo y no reescribe originales. Se mantiene manifiesto de backup v2 con versión SQLite independiente. Las escrituras de valores/membresías incrementan revisión/historial y reindexan FTS en la misma transacción; fallo revierte todo. Esquema usa revisión de colección y vista tiene su revisión propia.

Límites iniciales: 128 propiedades/patch, 10.000 miembros por colección; query evalúa/sortea candidatos en Rust, páginas de 1–100 (default 50), cursor offset hasta 10.000 sin snapshot multipágina. Desktop presenta una página de 50, no acumula filas sin límite. Costo de lectura crece con propiedades/documentos; no hay un benchmark de colecciones ni promesa de latencia. SQL más selectivo/virtualización se evaluarán con evidencia. [Fórmulas/rollups (3b)](formulas.md) agregan cálculos sólo de lectura; [vistas avanzadas (primer tramo 3c)](views.md) conserva cobertura por página. Templates/bases incrustadas siguen pendientes.
