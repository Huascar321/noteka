# Descubrimiento y baseline de planificación

Fecha: 15/10/2026. **Sólo planificación; ninguna tarea implementada o verificada.**

## Evidencia local observada

- `attachment_search_contracts.rs:46–59,89–113`: enums de extractor/localizador
  sólo UTF-8/PDF. La ampliación afecta index/search y tipos referidos, no requiere
  un comando/tool nuevo. No prometer igualdad de esos dos schemas.
- `attachment_search_storage.rs:214–255,314–345`: canonical rechaza XLSX; el hash
  v1 incluye manifiesto PDF/null, generación, fuente y fragmentos/localizadores.
  Añadir manifiesto XLSX debe mantener exactamente los bytes anteriores.
- `xlsx_storage.rs:87–163`: preflight de chunks/bytes/owner/parser, reconstrucción,
  hash, semántica y canonicalidad; devuelve Workbook validado sin ZIP/XML ni helper.
  `extraction_storage.rs:79–90` liga scope a workspace/ruta, no a versión SQLite.
- `attachment_search.rs:245–346,349–388,492–564`: put prepara fuera y revalida
  bajo escritor, rowid stride 512; reset borra índice reconocido y search verifica
  índice/candidatos/score y originales incluso del lookahead antes de responder.
- `attachment_search_storage.rs:606–753,765–841`: verify cubre todos los entries,
  copias y postings; coverage distingue configuración compatible sin proyectar
  todo lo no seleccionado; cursor v1 liga scope/query/filter/profile/revision.
- `migrations/010.sql`: tablas genéricas, 512 fragmentos/1 MiB por entrada y perfil
  literal_unicode61_v1. No hace falta ampliar DDL para XLSX, pero v11 antiguo
  reconoce el perfil/reset sin conocer la proyección nueva: se propone v12 como
  barrera de compatibilidad, no por necesitar una tabla nueva.
- `workspace.rs:283–307,319–368` y `batch_storage.rs:169–194`: open/migrate con
  backup/transacción, registry/snapshot y límite 64 MiB; incluir 012 sin cambiar
  las once migraciones anteriores ni la whitelist.
- Main `local-extraction` exige aún rechazo explícito XLSX. Cambiarlo requiere
  delta MODIFIED, no eliminar el rechazo como una corrección interna. También se
  reconcilian escenarios de versión y promesas de igualdad de contratos de main,
  incluido «Producto existente conservado» del lector independiente: la ampliación
  explícita de index/search no puede convivir con prometer que sus schemas son iguales.

Explorer aportó mapa parcial, verificado por lectura de las fronteras anteriores.
La duda perfil vs versión se resolvió en design: perfil/tokenización/hash heredados
se conservan y v12 protege clientes viejos. No se delegó implementación ni pruebas.

## Documentación consultada

SQLite FTS5 oficial, [phrases y bm25](https://www.sqlite.org/fts5.html), consultado
mediante Context7 `/websites/sqlite_docs` el 15/10/2026: las frases coinciden sobre
tokens del documento/fila y BM25 usa estadísticas globales, con menor score mejor.
No cambia el tokenizer existente a partir de un ejemplo de documentación. Los
cortes entre filas limitan coincidencias; el corpus propio debe probar el efecto.
La documentación no demuestra latencia ni sustituye la prueba del SQLite fijado.

## Baseline independiente

`/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/noteka-xlsx-search-plan.diMsYO/`:
`baseline.json`/`baseline/` con **369 archivos**, `binary-baseline.json` y `bin/`
con **ocho ejecutables debug/release**, `catalog-baseline.json` v1/52 y snapshots
de specs/instrucciones. No se creó v11 rebajando el binario futuro ni se ejecutó
un test de producto para declarar esta planificación aprobada. La extracción
archivada conserva su evidencia histórica independiente.

No hay commit inicial: al aplicar, contrastar esos hashes/copias, no un diff contra
Git vacío. Datos de ejecución/backup/modelos/credenciales quedan fuera de Git.
Home mantiene sus ocho tareas nativas pendientes; no se cambia ni sincroniza.

## Revisión documental

La planificación consta de cinco deltas, **14 requisitos/49 escenarios** (diez
requisitos modificados y cuatro añadidos), con los **32 nombres de escenarios
heredados** conservados y 75 requisitos de esas cinco capabilities no afectados.
`tasks.md` contiene **19 tareas sin marcar**, cada una con validación/trazabilidad.
La revisión cotejó títulos y escenarios con main, incluyendo las promesas de
compatibilidad heredadas; no sincronizó ninguna spec propuesta.

Auditoría contra la copia independiente: **365/369 archivos previos idénticos**;
los cuatro cambios previos son sólo plan/workflow/review queue/config. Los diez
archivos nuevos pertenecen únicamente a este cambio. Código, tests, fixtures,
gates, main, artefactos de Home y ocho binarios de referencia permanecen iguales.
`planning-audit.json` y logs de validación quedan en el directorio externo anterior.

OpenSpec estricto valida documentos, no la aplicación. `status` puede presentar
`isComplete=true` al existir los cuatro artefactos aun con **0/19 tareas** según
`list`: no se usa ese flag como aceptación de producto. Un primer auditor suponía
lo contrario y falló; el diagnóstico y su corrección de interpretación están en
`planning-audit-attempts.md`, sin marcar tareas ni cambiar código. No se ejecutaron
pruebas de producto ni se obtuvieron nuevas mediciones durante esta planificación.
