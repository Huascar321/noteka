# Design

## Context

Ver `proposal.md`. Lectores existentes en API v1/52: `extraction.fragments`,
`extraction.pdf_fragments`, `extraction.xlsx_cells`, todos por job/limit/cursor,
revalidación completa de fuente/derivado y respuesta hasta 2 MiB. Baseline durable
439 archivos: `~/.local/share/noteka-verification/active-attachment-reader-path.txt`.
Incluye preparación en aceptación, todavía sin sincronizar/archivar; la integración
se revalida junto con preparación, sin cambiar Rust/CLI/corpus en ejecución. Home sigue independiente.

## Goals / Non-Goals

**Goals:** acceso a todo el derivado confirmado mediante páginas, con su contrato
tipado y sin otra autoridad de datos. No es solamente un excerpt/hit seleccionado.

**Non-Goals:** ver proposal. No agregar filtros por hoja/rango que la API no tiene,
ni una grilla virtual vacía o formato visual calculado a partir de estilos.

## Decisions

1. Panel independiente en editor de nota, ID de job manual y tipo explícito. Cada
tipo mapea a una operación, sin get/job list/descarga adicional ni heurística de
formato. Leer equivocado devuelve el error Rust; no encadena fallback. Alternativa
de mezclar lectores con Search descartada: éste sigue buscando snapshots elegidos.
2. Controlador con unión discriminada de tres páginas, una página de veinte y
generación/cursor asociados a criterios confirmados. Editar invalida página/error;
siguiente conserva criterios y compara generación/job/tipo antes de publicar.
Inicio vuelve explícitamente a leer sin cursor. Respuesta defensiva de más de
veinte items o identidad incompatible se rechaza entera, no se trunca.
3. Misma estrategia probada de lifetime/cupo físico compartido uno; abortar la
espera no cancela IPC ya iniciado. No polling ni actualización por activación.
Los originales/SQLite siguen siendo autoridad; no caché persistente de contenido.
4. Render escapado completo. UTF-8 rangos byte/línea y pre literal; PDF página,
rango derivado, manifiesto completo con no_text y limitaciones. XLSX tarjetas
por celda (máximo veinte), hoja tomada del manifiesto por ID, A1/ordinal y cache
tipada/fórmula separadas. JSON literal para metadatos de fórmula y manifiestos,
sin conversión de número léxico/date_iso ni expansión de celdas/merges. Manifest
es completo aunque covers_generation sea falso para la página.
5. Integración mínima tras aceptación aislada, con revalidación conjunta de la UI de preparación, usando key workspace/
nota/epoch y sin cambiar guardias/Home. No es preciso guardar para consultar un
derivado, pero navegar protege el buffer igual que antes.
6. Cliente prueba identidad/páginas/errores/lifetime/cola. Headless usa fuentes
sintéticas y CLI real, golden independiente y snapshots para lecturas/buffers.
Núcleo, contratos, corpus y gates sin cambios: la aceptación recién ejecutada de
preparación puede reutilizarse con hashes completos/fechas/alcance explícitos;
no se repiten costosos oracles/gates si esos binarios/fuentes siguen idénticos.
Se ejecutan check/build, lectores/cliente nuevos y regresión headless pertinente.
Si aparece un cambio del núcleo o duda material, se vuelve a ejecutar el gate o
regresión afectada antes del cierre. No se usa evidencia histórica v11 perdida.

## Risks / Trade-offs

- [ID manual] → etiqueta/guía, error Rust explícito; refinamiento de navegación a
lectura puede proponerse después sin inventar asociaciones/metadata.
- [Respuesta larga] → veinte items y pre con scroll/wrap, nunca corte de texto.
- [Fuente/generación concurrentes] → lectores validan; cursor y comprobación de
identidad evitan mezclar; no reintento/reparación silenciosos.
- [Nueva UI durante aceptación anterior] → archivos inicialmente no importados; después, cambios App/puente identificados y toda la UI revalidada conjuntamente. Se conservan 207 fuentes anteriores y Rust/corpus/contratos íntegros.

## Migration Plan

Sin migración, dependencia, contrato ni helper nuevos. Integración reversible del
panel sin descartar fuentes/generaciones/índice. Sincronizar sólo los cuatro
requisitos verificados y conservar íntegros los requisitos previos, incluidos los
seis de preparación después de su cierre. No sincronizar ni archivar Home.
