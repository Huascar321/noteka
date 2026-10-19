# Search: evidencia de adjuntos locales

[`add-local-evidence-search-ui`](../openspec/changes/archive/2026-10-16-add-local-evidence-search-ui/proposal.md)
cerró **17/17 tareas**, con UI, regresión Rust/MCP y cuatro gates aprobados. Único
delta sincronizado y contrastado; cambio archivado. [Evidencia](verification-evidence-search-ui.md).

## Ámbitos y preparación

Search separa «Notas y entidades» de «Adjuntos»; no suma scores ni comparte
cursores/filtros. Notas conserva sus filtros y apertura del bloque original.
Adjuntos usa sólo `extraction.search`, sin consultas al montar, polling ni
descarga de originales/base64. Texto literal y filtros opcionales AND de
extractor, ID de trabajo y ID de adjunto; no hay filtro de entidad/archivo/contexto.

La preparación es explícita: importar → extraer → seleccionar generación con
revisión esperada. Está disponible desde «Preparar adjuntos locales» en una nota
o por CLI/MCP; ver [preparación UI](attachment-preparation-ui.md). El panel
«Leer generación local» permite recorrer el derivado sin selección en Search;
ver [lector UI](attachment-reader-ui.md). Ver [UTF-8/PDF](attachment-search.md) y
[XLSX](xlsx-search.md). Buscar no importa, encola, extrae, indexa ni repara nada.
Sin selección, cero hits es esperable; no implica ausencia en toda la base.

## Consultas y páginas

Rust conserva validación de 1–16 términos y 256 bytes UTF-8, UUIDs/cursor y errores.
La UI no corta texto ni confunde maxlength de caracteres con bytes; omite filtros
vacíos. Hay una página de hasta 20 hits por vez: siguiente reemplaza, no acumula.
Volver al inicio o buscar de nuevo es explícito. Editar criterios elimina página,
detalle y cursor sin consultar; fallo retira la evidencia anterior y conserva
código/razón, nunca se muestra como «sin coincidencias» ni reinicia solo.

El controlador comparte un cupo físico entre instancias. Abortar la espera lógica
o salir del panel descarta éxito/error tardíos, **no cancela invoke/Rust**: el
cupo permanece ocupado hasta completar el IPC. La espera cancelada en cola no
inicia otra llamada. Sin bloqueos globales de Home/editor ni contenido en logs o
almacenamiento persistente del navegador.

## Evidencia y límites

Texto canónico completo, con espacios/saltos y contenido no confiable escapado.
El detalle muestra IDs, hashes, generación, extractor/versión, ordinal y flag
anterior; no infiere nombre, dueño ni asociación de entidad. Desplegar no abre ni
revalida el archivo: describe el snapshot verificado al consultar.

- UTF-8: bytes `[inicio,fin)` y líneas del original, no índices JavaScript.
- PDF: página y bytes derivados, con scope/limitaciones, sin geometría/OCR/tablas.
- XLSX: hoja ID/A1/ordinal de celda, campo/proyección/bytes totales/rango; fórmula
  y caché separadas y porciones explícitas. No recálculo ni offsets ZIP/JSON.
  Las hojas ocultas pueden aportar hits; el hit no incluye su nombre/visibilidad.

Cobertura filtrada del núcleo: generaciones seleccionadas/sin indexar/incompatibles,
jobs sin generación, fragmentos/bytes, versión/perfil/revisión. Integridad completa
del índice no equivale a comprobar todos los originales: search verifica candidatos
de página/lookahead. Los contadores no son total de coincidencias; selección sin
texto no prueba OCR o libro visualmente vacío. No garantiza frescura posterior.

Teclado con controles nombrados y detalle nativo plegable; ambos temas. La lista
estrecha hace extensos algunos hashes/rangos: se ajustan, no se truncan. Buscar o
alternar ámbito no reemplaza el buffer. Abrir otra entidad o navegar conserva
guardar/descartar/cancelar. Sin modelos, remotos, cambios de Home ni aceptación
nativa implícita por pruebas del puente headless.
