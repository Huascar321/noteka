# Vistas avanzadas: primer tramo de 3c

Las vistas consultan entidades y propiedades compartidas. Tablero, calendario y galería no crean páginas nuevas al presentar registros. El núcleo Rust valida filtros y calcula grupos/localizadores en el mismo snapshot SQLite de la consulta; el cliente sólo renderiza. [Templates](templates.md) y [bases incrustadas](embedded-collections.md) se incorporan en el segundo tramo de 3c; el [Home de referencia](home-reference.md) sigue siendo otra entrega.

## Contrato compartido

`view.save` conserva `{collection_id,name,layout,config,id?,expected_revision?}`. Actualizar exige la revisión de la **vista**, no la colección; nombre/layout/config confirmados se conservan ante conflicto. Layouts: `table`, `list`, `board`, `calendar`, `gallery`.

`database.query` admite vista guardada (`view_id`) o config/layout ad-hoc; no ambos. Layout ad-hoc default table. Salida: `properties`, `config`, `layout`, `records`, `matching_count`, `groups`. Un grupo contiene `key`, `label`, `entity_ids`, no copias de registros. `matching_count` cuenta registros coincidentes antes de paginar, no apariciones de un rango en días.

Páginas 1–100 (default/desktop 50), máximo 10.000 candidatos, orden de filtros/sorts existente y desempate ID. Grupos/días cubren **sólo la página actual**; una columna vacía en una página no indica ausencia en toda la colección. Siguiente/inicio conserva consulta, sin snapshot garantizado entre páginas con escrituras externas. No hay benchmark de esta entrega.

## Filtros compuestos

`filters` existentes (hasta 16) se combinan AND con `filter_tree` opcional. Árbol estricto:

- `{op:"condition",filter:{property_id,op,value?}}`
- `{op:"all",children:[...]}` / `{op:"any",children:[...]}`: AND/OR, no admiten grupos vacíos.
- `{op:"not",child:...}`: negación booleana.

Hasta 64 nodos, profundidad 8 y 32 condiciones totales (legacy + árbol). Todo el árbol se valida, incluidas ramas cortocircuitadas. Referenciar un cálculo fallido en cualquier condición/orden rechaza query, aunque otra rama lo excluiría. Guardar valida configuración/tipos; no ejecuta todos los cálculos de datos al definir una vista.

Operadores: eq/ne, lt/le/gt/ge, contains, is_empty. Eq/ne comparan valores tipados; decimales/moneda con i128 exacto, no floats. Missing se lee null; ne niega eq e incluye null si se compara contra un valor no null. Ordenados dan false para celda null y no admiten value null. Admiten number/decimal/money o texto escalar (incluidos fechas/timestamps/selecciones y fórmulas number/decimal/text), no checkbox, listas, rangos ni relaciones. Texto usa orden binario Unicode; fechas civiles YYYY-MM-DD conservan orden civil. Timestamps son strings UTC de los originales y se comparan como texto: usá el mismo formato UTC del contrato, no un offset como sustituto de una conversión temporal.

Contains distingue mayúsculas y conserva los tipos textuales de 3a/3b; is_empty incluye null/string/array vacío, no false/cero. NOT de una comparación falsa por null es true: no implementa lógica ternaria SQL. No hay SQL ni código ejecutable cliente.

```json
{
  "filter_tree": {
    "op": "all",
    "children": [
      {"op":"condition","filter":{"property_id":"id-importe","op":"ge","value":"48.570"}},
      {"op":"not","child":{"op":"condition","filter":{"property_id":"id-pago","op":"eq","value":true}}}
    ]
  }
}
```

## Tablero

```json
{"group_by":"id-select-o-checkbox","columns":["id-titulo","id-importe"]}
```

`board` exige group_by de esta colección y tipo select/checkbox. Opciones de select en orden declarado, incluidas vacías; checkbox false/true; grupo separado Sin valor (key null). No se agrupa por relaciones/built-in payload directamente. Editar un patch confirmado o actualizar desde CLI mueve el mismo ID al próximo snapshot. No hay arrastre que escriba valores ni progresos ficticios.

## Calendario civil

```json
{"calendar":{"property_id":"id-date-o-date-range","month":"2026-10"},"columns":["id-titulo"]}
```

`calendar` exige date/date_range propio y mes explícito YYYY-MM, años 0001–9999. No lo elige por la zona del sistema, no convierte fechas a UTC y no crea recurrencias. Fechas fuera del mes se excluyen; rangos solapados se recortan al mes, referencian el mismo ID en sus días. Siempre incluye grupo Sin fecha para null/missing y los días vacíos. Máximo 31 días + grupo sin fecha; los registros planos son únicos.

Configuración ajena al layout se rechaza: board no admite calendar, calendar no admite group_by, table/list/gallery no admiten ninguno. Cambiar de layout requiere config compatible, no conversión silenciosa de vistas originales.

## Galería y escritorio

Gallery usa tarjetas textuales con columnas seleccionadas, errores calculados y acciones Abrir/Editar valores. No descarga previews externos ni invoca attachment.read automáticamente; People con fotos del Home necesita otra entrega.

`CollectionPresentation.svelte` comparte los cinco layouts y cobertura/errores a partir de `DatabasePage`. DatabasePanel conserva consultas/escrituras/buffers y ofrece el callback opcional de edición; el lector incrustado sólo ofrece apertura del original. Ninguno recalcula filtros/grupos en TypeScript.

Collections → Nueva vista → Layout y config JSON por IDs. Tabla/lista/galería también pueden presentarse temporalmente; ese cambio es consulta ad-hoc con los filtros de la vista, sin su group_by/calendar y sin guardarla. Para tablero/calendario, guardá una vista configurada. El selector de vista permite volver a su layout persistido.

El layout del formulario es distinto del snapshot visible: elegir otro no relabela las filas confirmadas. Cancelar navegación/cierre conserva JSON/layout, conflicto no rebasa revisión, Recuperar buffer incluye layout/ID/revisión de vista. Si una consulta/refresco falla, los resultados anteriores se retiran, no aparecen como pertenecientes a la nueva vista. Si guardar confirmó pero refresh falló, se comunica el commit confirmado.

## Esquema y recuperación

La migración 006 (SQLite v6) incrementó versión de formato **sin** reescribir JSON antiguo ni agregar tablas vacías, con backup previo de v5 y rollback. La 007 agrega templates en v7, también sin reescribir vistas. Una recuperación selecciona el backup anterior con cliente compatible, no downgrade inplace. Manifiesto de backup sigue formato v2; ver [recuperación de templates](templates.md).

Config/layout futuros desconocidos impiden su lectura/edición reconocida; no se convierten para sobrescribirlos. Backup/restore conserva JSON crudo y originales; no existe un editor de reparación de formatos futuros. Vistas tabla/lista antiguas leen nuevos campos con defaults sin modificar los bytes almacenados durante migración. [Colecciones](collections.md), [fórmulas](formulas.md).
