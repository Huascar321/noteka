# Revisión de tres recortes de interfaz de 5a

Revisión final del 17/10/2026, con aceptación técnica aprobada. Se revisaron
preparación (6 requisitos/13 escenarios), lector (4/8) y búsqueda agrupada (4/10,
tres escenarios heredados). Trece requisitos añadidos y uno modificado; no se
adoptan requisitos futuros de OCR, formatos, embeddings o agentes.

## Completitud y coherencia

Las matrices de [preparación](verification-attachment-preparation-scenarios.md),
[lector](verification-attachment-reader-scenarios.md) y
[agrupada](verification-grouped-search.md) contrastan cada escenario con código,
tests de cliente y CLI/SQLite reales. El controlador de preparación usa CAS de
job/control/índice y generación observados, retira inspección tras escritura/error
y no repite un resultado incierto. Los tres paneles evitan llamadas al montar o
editar criterios; los cupos físicos y lifetimes conservan IPC iniciado y descartan
resultados tardíos. Las guardias de App protegen el buffer; abrir otra nota no
invalida por sí mismo la identidad de una consulta global.

Texto, procedencia y manifiestos se presentan completos y escapados. El lector
usa una página de veinte sin convertir números, ejecutar fórmulas o atribuir
offsets ZIP/originales al derivado. Los grupos de búsqueda conservan sus errores,
páginas, scores y cobertura independientes. No hay ranking/snapshot/total global
inventado ni preparación automática desde Search. Las decisiones del diseño
coinciden con el producto probado y no requieren un ADR persistente nuevo.

## Scope y evidencia

La comparación usa tar/SHA-256 del baseline previo, no HEAD vacío. Los 424 archivos
originales siguen presentes. Los 151 protegidos son idénticos: Rust, contratos,
corpus/goldens, gates, host Rust y todos los archivos de Home. Los cambios de App
se limitan a imports/montajes de paneles, tercera opción Search y sus ramas de
loadList/refresh; no cambian guardias, composición o navegación Home. Tauri sólo
cambia el hook de preparación y añade el helper XLSX ya existente al staging.
Package sólo agrega scripts, sin dependencias o lock nuevo.

La reparación UTF-8 y traducción underscore/guion pertenecen al puente headless;
se comprobaron con test determinista de bytes fragmentados y regresión anterior
intacta. Los harnesses contienen únicamente fixtures sintéticas externas. Los
tests/generadores previos no pierden asserts ni cambian sus esperados.

[Aceptación conjunta](verification-local-preparation-closure.md): 36 etapas
frescas compartidas, 287 Rust por perfil, 75 cliente/editor, 11 puente/lotes,
3.116 checks UI, contratos completos idénticos y cuatro gates finales aprobados
con cuotas/validación/targets intactos. Los 222 pins del producto probado se
contrastaron al cierre. El gate fallido nuevo y el histórico permanecen visibles;
sus causas no se consideran demostradas ni se afirma optimización.

## Límites y cierre

IDs manuales y paneles de datos son UX deliberadamente inicial. El warning Vite
previo sigue visible. Headless/host no-bundle no certifican IPC/foco/cierre nativo,
firma o distribución. Home permanece activo 9/17. No hay migración, modelos,
proveedores, OAuth, pagos, publicaciones ni cambios de originales.

No quedan defectos materiales demostrados o escenarios sin evidencia en estos
tres recortes. Sync añadió sólo sus requisitos verificados: local-extraction
22→28→32 y local-text-search 20→23, preservando todos los bloques ajenos y los tres
escenarios heredados de búsqueda. Se archivaron separadamente después de comprobar
esa igualdad; Home no participa. **La fase 5a continúa con OCR y formatos.**
