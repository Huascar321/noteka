# ADR 0023: evidencia PDFKit seleccionada por página

Estado: adoptado y verificado, 20/10/2026.
[Evidencia](../verification-pdfkit-search-ui.md).
Cambio `add-local-pdfkit-search-ui`.

## Context

El backend14 guarda Document v1 íntegro y lo lee sin helper; FTS todavía rechaza
ese tipo y los paneles no lo ofrecen. Cliente14/checkpoint y fixture real de cinco
generaciones/cuatro selecciones/41 tablas fueron conservados antes de015. Backup
/restauración14 tienen filas y originales exactos; no recuperan la evidencia v11.

## Decision

Proyectar sólo texto canónico por página física/rango UTF-8, con un locator nuevo
que identifica perfil, OS/build y limitaciones. Fragmentos<=4096 bytes,512/1MiB
por selección, sin cruzar páginas ni indexar metadata. El hash content.v1 recibe
manifiesto del formato nuevo sin modificar hashes anteriores. Se preservan todos
los chequeos de fuente/canonicalidad/postings/CAS y los gates300/150.

015 es una barrera de versión sin DDL de datos: cliente14 no debe interpretar un
índice nuevo que sus tablas genéricas pueden almacenar. Backup14 y transacción,
rollback y rechazo futuro16 se verifican con clientes independientes, sin helpers
al migrar. No se borra/reindexa nada; sólo la selección posterior usa la proyección.

Preparación/lector/Search existentes incorporan un caso explícito, conservando
operaciones, cupos/lifetimes, buffers y guardias. Home/App/tema no cambian.
Alternativas rechazadas: indexar JSON, fingir el perfil PDF estricto, derivar
geometría/orden visual del texto o iniciar extracción/indexación al buscar.

## Consequences

Se necesitan pruebas de compatibilidad52 contratos no afectados y datos14 reales,
fronteras/corrupción, E2E nativo y UI headless. Los locators son mayores; se conserva
el cap2MiB y se informa exceso completo. No implica OCR dePDF, tablas, toda5a,
modelos/proveedores, fase6 o aceptación nativa del host.
