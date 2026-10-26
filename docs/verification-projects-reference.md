# Revisión acotada de Projects — 23/10/2026

## Alcance

Cambio `match-home-project-reference`: tablero Projects en Home, procedencia y
orden opcionales, resúmenes Rust y recuperación explícita de vínculos/arte. El
checkout previo se conservó; baseline, fuentes e informes privados viven en
`~/.local/share/noteka-verification/2026-10-23-project-reference`, fuera de Git.

## Recuperación

Se verificó la secuencia completa de 578 Tasks y 20 Projects del CSV/manifiesto.
Cada ID original se contrastó con título/prefijo largo y Created/Edited/Status
del bloque inicial de propiedades del Markdown. Las coincidencias son únicas;
el caso sin título conserva el fallback del importador. No se leyeron cuerpos.

Los metadatos actuales de los veinte proyectos se obtuvieron del conector
Notion, limitados a identidad, estado, archivo, fechas y relaciones. Las doce
tarjetas visibles acreditan orden e imágenes. La consulta de tareas sólo devolvió
478 metadatos antes de alcanzar la cuota de SQL del conector; no se reintentó
la consulta rechazada ni se atribuye cobertura completa a esas filas. La
identificación de las 578 tareas se resolvió con el export y el manifiesto.

- 514 vínculos Task→Project resolubles: 14 previos y **500 añadidos**.
- **17** Project→Tag y **5** GoalNote→Project añadidos.
- **12** asignaciones de icono original; los ya elegidos se conservaron.
- **20** procedencias de proyecto; ocho archivadas en origen y doce visibles.
- Siete referencias Task de la fuente no aparecen en el export/manifiesto y se
  omitieron con informe. No se crearon tareas parciales ni estados ficticios.
- Cero diferencias entre estados locales de proyecto y metadatos consultados.
- Backup canónico previo, CAS e informe incremental con revisiones/payloads.
  Los **1.802 IDs** y el digest de todos los documentos permanecieron idénticos.
  Los estados y fechas de tareas se conservaron; las relaciones incrementan sus
  revisiones normalmente.

536 operaciones de relación resultaron en 522 vínculos nuevos y 14 idempotentes.
Las fuentes, cuerpos originales, imágenes y backups no se incorporan a Git.
La operación es recuperable mediante informe y revisión vigente, sin promesa de
atomicidad global ni restauración ciega sobre cambios posteriores.

## Compilación y presentación

`cargo build --locked --offline -p noteka-cli` y generación de contratos aprobados.
`npm run build` final aprobado con cero errores y cero advertencias Svelte.
Se conserva la advertencia previa de Vite por bundle mayor a 500 kB.
Bundle revisado: `index-Cx8WP_3b.js`, 691,62 kB sin gzip.

Inspección manual asistida con Chrome headless, UI de producción y puente de
sólo lectura a CLI Rust sobre copia del workspace real. Se revisaron encabezado,
New Project, pestañas, columnas, tarjetas, iconos, contexto y resúmenes en claro
y oscuro. Board inicial muestra doce tarjetas, cinco columnas de 196 px, 27
imágenes locales y quince referencias de contexto. Active: siete tarjetas/dos
columnas; Planned: cero tarjetas/una columna. Las columnas vacías conservan
New page.

Porcentajes observados en orden del tablero: 66,7; 18,2; 97,4; 90,5; 100; 100;
100; 100; 70,2; 95; 28,6 y 93,8. Coinciden con las doce tarjetas de Notion en este
snapshot, pero se calculan desde relaciones/estados locales. Los contadores
pendientes/vencidos también corresponden a los datos locales, no a strings
copiados. El día civil usado es 23/10/2026, `America/La_Paz`.

Viewport de 420 px: overflow global cero; el tablero tiene 1.030 px de contenido
y 372 px visibles, con desplazamiento interno de 658 px hasta el extremo.
Cero errores de ejecución del navegador y cero requests externos durante la
inspección. Los controles son botones nativos y los lectores mantienen las
guardias, límites, cursores, error/reintento y descarte tardío existentes.

La revisión de código contrastó captura con estado de columna, protección de
buffers y apertura de referencias con sus callbacks existentes. No se guardaron
proyectos de prueba. La primera compilación produjo una advertencia Svelte de
tabindex en contenedor no interactivo; se retiró ese tabindex, conservando
navegación por botones y scroll interno. El log previo se conserva.

## Límites y evidencia

No se añadieron ni ejecutaron suites no solicitadas. No se repitió aceptación
nativa Tauri/WebKit o cierre de ventana. No se implementó arrastrar tarjetas,
sincronización, todas las fórmulas de Notion ni el dominio Goals. Los payloads
enriquecidos requieren un cliente actualizado para edición con esquema cerrado.

Logs ignorados: `projects-cargo.log`, `projects-contracts.log`,
`projects-build-final.log`, `projects-recovery-apply.log`,
`projects-preview-accepted.log`; capturas `projects-reference-{light,dark,narrow}.png`.
Los intentos de dry-run con referencias sin identidad detuvieron la ejecución
antes de cualquier escritura; el alcance se corrigió para informar las siete
referencias ausentes y conservar la recuperación acreditada. Los originales de
esos intentos permanecen disponibles y no se presentan como evidencia final.

## OpenSpec

Cuatro tareas; tres requisitos nuevos desktop-shell y uno knowledge-entities.
El cierre sincroniza sólo estos requisitos implementados y conserva los bloques
previos y las otras quince specs. La validación documental no reemplaza la
inspección de aplicación descrita arriba.

Cierre: **4/4 tareas**, deltas contrastados y archivo
`2026-10-23-match-home-project-reference`. Se preservaron exactamente los 28
requisitos desktop-shell y nueve knowledge-entities anteriores; ahora son 31 y
diez. Las otras quince specs conservaron sus hashes. Validación estricta
documental aprobada, con el alcance técnico acotado indicado arriba.
