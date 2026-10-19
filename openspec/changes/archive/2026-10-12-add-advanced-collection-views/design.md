# Design

## Context

`database_contracts.rs` expone tabla/lista, hasta 16 filtros AND y cuatro órdenes. `database.rs` valida config, hidrata candidatos/calculadas en un snapshot, filtra/sortea y pagina (50 default, 1–100, máximo 10.000 miembros). `DatabasePanel.svelte` guarda config JSON/IDs y protege buffers; los grupos todavía no existen. Vistas viven como JSON en `views`; el SQLite actual es v5 y `calculation_errors` no se persiste. Tests existentes verifican decimal exacto, null, paginación, revisión y paridad. Motivación/alcance: proposal.

## Goals / Non-Goals

**Goals:** reutilizar lectura/validación/revisión, mantener IDs y cálculos originales, separar reglas Rust de rendering y acotar costo/memoria/paginación.

**Non-Goals:** cambiar documentos v1/v2, agregar tablas de templates o nodos de bases incrustadas, arrastre que modifique valores, previews automáticos/URLs, nuevo motor SQL o virtualización. Galería inicial textual es una decisión visible, no una promesa de imágenes ya implementadas.

## Decisions

### 1. Filtros con AST pequeño y semántica explícita

Extender FilterOp con ne/lt/le/gt/ge. `filter_tree` opcional de ViewConfig: `{op:"condition",filter:PropertyFilter}`, `{op:"all"|"any",children:[...]}` y `{op:"not",child:...}`. Los filtros legacy se aplican AND con el árbol. Máximo legacy 16, árbol 64 nodos/profundidad 8, 32 condiciones totales combinadas; grupos all/any vacíos rechazados. Campos desconocidos rechazados antes de evaluar.

Eq/ne conservan comparación tipada existente, incluyendo null (missing se lee null); ne niega eq. Contains conserva sensibilidad a mayúsculas; is_empty incluye null/string/array vacío, no false/cero. Ordenados requieren valor y celda no null y propiedad numérica/decimal/moneda o escalar textual (title/text/url/email/select/date/timestamps; fórmulas number/decimal/text). No ordenar checkbox/listas/rangos/relaciones mediante estos comparadores. Dinero usa el comparador decimal existente, no casts cliente. Not es negación booleana: negar un comparador falso por null devuelve true; se documentará explícitamente.

Validar toda rama aunque no se evalúe; recolectar todos los IDs de condiciones para rechazar errores calculados antes de filtrar, como los filtros actuales. Mantener validación de datos desconocidos/corruptos; no convertirlos a cero. Preferir helpers de validación/selección compartidos a replicar reglas en Svelte o generar SQL arbitrario.

### 2. Configuración y proyección de presentación Rust

Agregar layouts board/calendar/gallery. ViewConfig agrega `group_by` opcional y `calendar` opcional `{property_id,month:"YYYY-MM"}`. Board requiere group_by select/checkbox propio; calendar requiere date/date_range propio y mes válido (años 0001–9999). Configuración ajena al layout se rechaza al guardar/consultar; table/list/gallery no requieren esas opciones. Vistas antiguas sin nuevos campos mantienen defaults.

DatabaseQuery admite `layout` opcional para consulta ad-hoc (default table); `view_id` excluye layout/config. DatabasePage devuelve layout, config, matching_count y `groups` con key/label/entity_ids; conserva records paginados completos. Proyección usa IDs de la página, no copia RecordData por grupo. Tablero mantiene opciones declaradas incluso vacías, checkbox false/true y grupo null separado; orden dentro de grupo sigue el query.

Calendar filtra previamente fechas/rangos que solapan el mes e incluye null en grupo Sin fecha. Proyecta todos los días civiles del mes, recorta rangos al mes y repite sólo referencias al mismo ID en días relevantes. No convierte fechas a instantes ni elige mes según timezone del sistema. Año 9999/mes diciembre requiere cálculo seguro del último día sin desbordar año; días civiles pueden iterarse con chrono ya existente. Los records planos siguen únicos; matching_count cuenta registros, no apariciones por día.

Gallery presenta tarjetas con columns y errores, sin pedir attachment.read ni fetch automático. List/table existentes conservan formato y apertura del original. Alternativa descartada: cargar toda colección o agrupar en frontend, porque mezclaría reglas y límites entre CLI/UI.

### 3. UI protegida y cobertura honesta

Reutilizar panel/formulario y guardia de buffer; agregar nuevos layouts y ejemplos JSON de agrupación/calendario/filtro. Componentes/snippets Svelte 5 keyed por ID para tarjetas/grupos; documentación oficial de snippets/each consultada por Context7 el 12/10/2026. Sin librería de calendario o tablero nueva.

La UI indica "Página actual: N de M registros" en todas las presentaciones y permite página siguiente/inicio. Grupos vacíos de una página no significan colección completa vacía. No acumular registros de todas las páginas ni afirmar snapshot multipágina. Abrir/editar preserva la guardia existente.

Separar estado de layout/config confirmado de selección/formulario pendiente. Al refrescar o cambiar vista, publicar layout/config/records juntos sólo tras query exitosa; si falla, mostrar error y limpiar resultados o conservarlos claramente bajo la configuración anterior, no bajo una nueva. El layout elegido para un buffer no debe relabelar silenciosamente resultados visibles. Guardar confirmado seguido de refresh fallido conserva mensaje de commit confirmado.

### 4. Evolución de formato con recuperación

SQLite v6/migración 006 sólo incrementa versión; no reescribe JSON ni valores. Es un cambio real de formato de vistas/DTOs aunque no añada tabla: clientes viejos rechazan v6 antes de tocarlo. Nuevos clientes leen defaults de v5, migran con backup previo y conservan IDs/config AST. Un formato futuro unknown no se edita/deserializa de forma lossy para hacerlo legible. Backup crudo conserva JSON desconocido y la restauración puede demostrar preservación; la UI puede mostrar error en vez de editor de reparación.

## Risks / Trade-offs

- Agrupamientos parciales por página → matching_count y etiqueta explícita; pruebas de recorrido sin duplicación/pérdida. No contar sólo tarjetas como cobertura total.
- Rangos largos o meses extremos → recorte civil máximo 31 días y comprobaciones de límites, incluida fixture bisiesta.
- Trees o errores ocultos en OR → validación completa/límites y precheck de todos los cálculos referenciados.
- Frontend atribuye snapshot viejo a vista nueva → publicación conjunta, prueba de query fallida y cambios de selección/refresh.
- JSON/IDs poco amigables → contrato/ejemplos; refinamientos de UX se revisan sin bloquear núcleo verificable.
- Query actual carga hasta 10.000 candidatos → conservar límite, no añadir claims de rendimiento sin benchmark; proyección sólo de la página.

## Migration Plan

1. Añadir 006 y registrar v6 en create/open/validación. Sin otras tablas/columnas ni backfill.
2. Backup v5 antes de transacción; inyección de fallo demuestra v5 intacto y backup restaurable. Evolucionar fixtures de migración existentes sin eliminar sus casos.
3. Leer config antigua con defaults y probar byte-preservation de JSON original, datos/documentos/FTS/AST y backups.
4. Probar campos/layout futuros desconocidos: lectura/edición rechaza, backup/restore conserva crudo. Clientes antiguos no deben abrir v6; recuperación hacia v5 es selección del backup anterior, no downgrade inplace.
5. Verificar Rust/CLI/contratos/frontend/nativo offline/release/OpenSpec y revisar diff antes de sincronizar/archivar. 3c sigue pendiente de templates/bases incrustadas.
