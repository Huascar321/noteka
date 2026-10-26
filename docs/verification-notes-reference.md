# Revisión acotada de Notes — 23/10/2026

## Alcance

Cambio `match-home-notes-reference`: Notes en Home, clasificación/favorito/fuente
opcionales, proyecciones Rust y recuperación explícita de metadata/arte. Se
conservó el checkout con sus cambios anteriores. Baseline y evidencias viven en
`~/.local/share/noteka-verification/2026-10-23-notes-reference`, fuera de Git.

## Datos recuperados

- 1.172 páginas conservan procedencia; 195 corresponden a registros de la base
  Notes. Seis páginas auxiliares dentro de su carpeta carecen de fila CSV
  inequívoca y permanecen como Pages, accesibles en All.
- Tipos/favoritos originales recuperados. Los dos nombres duplicados se
  resolvieron con Created/Updated y propiedades del bloque inicial exportado,
  sin elegir por posición ni leer el cuerpo.
- 16 vínculos nota/tag o nota/proyecto añadidos de las diez filas visibles.
  No se atribuyen vínculos recuperados a las otras 185 notas.
- Nueve asignaciones de imagen: dos notas, cuatro tags y tres proyectos.
  Ocho iconos nativos de nota usan Lucide; el icono nativo de contexto
  reutiliza el mismo adjunto ya acreditado para su recurso original.
- Los metadatos actuales de diez registros se consultaron por ID mediante el
  conector Notion, seleccionando sólo fechas/tipo/favorito/archivo. Dos fechas
  del export eran antiguas y se actualizaron en la procedencia. No se pidieron
  cuerpos ni se escribieron páginas remotas.
- Backups canónicos e informes incrementales conservan payloads/revisiones
  anteriores. Los 1.802 IDs y el digest de todos los documentos permanecen
  idénticos antes/después. Las actualizaciones usan CAS normal y no falsifican
  timestamps locales ni cambian documentos, títulos, estados o fechas civiles.

## Compilación y presentación

`cargo build --locked --offline -p noteka-cli` aprobado. Contratos regenerados y
`npm run build` aprobado con cero errores/advertencias Svelte. Vite mantiene la
advertencia de bundle mayor a 500 kB.

La inspección usa Chrome headless con UI de producción, un puente a CLI Rust y
un snapshot del workspace real. El puente permite sólo lecturas de producto.
En claro/oscuro se revisaron encabezado, ocho vistas, filas, imágenes, relaciones,
tipo y antigüedad. El primer acceso a Notes carga diez filas; Load more conserva
las diez anteriores y llega a veinte. Cambiar de vista reinicia la página.

Coberturas observadas: Notes: 132; Fav.: 21; Clips: 50; Voice: 0; Journal: 2; Meetings: 9;
All: 1.172 e Inbox: 0. Voice/Inbox muestran vacío real. Los conteos corresponden a
este snapshot; pueden cambiar con ediciones posteriores.

Viewport de 420 px: cero overflow global; títulos se distribuyen en líneas, metadatos
se envuelven y las vistas conservan scroll horizontal. Cero errores de ejecución
del navegador y cero requests externos durante el recorrido. Los recursos
personalizados se obtienen como adjuntos locales. Se revisó el código de limpieza,
invalidación y retry del lector; los demás lectores conservan su tamaño previo.

La primera captura mostró dos fechas antiguas del export y un icono nativo
genérico; se recuperó metadata actual y la equivalencia del icono. Una ejecución
de inspección usó un selector ambiguo para Notes; se acotó al grupo de vistas.
Los logs anteriores se conservan, sin acreditarlos como aceptación final.

## Límites

No se añadieron ni ejecutaron suites no solicitadas. Las verificaciones históricas
de navegación/Tasks/Home conservan su alcance. No se ejecutó una nueva aceptación
Tauri/WebKit ni cierre de ventana nativa. No se acredita sincronización continua,
grabación/transcripción de voz, filtros/automatizaciones completos de Notion ni
recuperación de todos los vínculos de la base.

Los días usan fecha civil del workspace, por lo que pueden diferir de la fórmula
relativa almacenada en Notion. Los timestamps exactos de fuente siguen disponibles.
Source es metadata de lectura; tipo/favorito se editan con las guardias existentes.


## Evidencia final y OpenSpec

Bundle aceptado: `index-Dkt5SuJD.js`. Logs ignorados: `notes-build-accepted.log`,
`notes-preview-accepted.log`, `notes-recovery-apply.log`, `notes-live-apply.log`
y `notes-native-apply.log`; capturas `notes-reference-{light,dark,narrow}.png`.
El snapshot final está separado del workspace personal y el navegador usa sólo
lecturas. Sus 18 imágenes incluyen todos los iconos recuperados visibles.

Cuatro requisitos añadidos: tres desktop-shell y uno knowledge-entities. Los
25/8 requisitos previos se conservaron y las otras 15 specs no cambiaron durante
la sincronización. Validación estricta documental aprobada; no sustituye la
revisión de aplicación descrita arriba.

OpenSpec: 5/5 tareas cerradas, deltas contrastados con main y archivo
`2026-10-23-match-home-notes-reference`.
