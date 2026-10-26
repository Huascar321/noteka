# Revisión acotada de People — 24/10/2026

## Alcance y datos

`match-home-people-reference`: propiedades opcionales, validación y proyecciones
Rust; galería, listas, calendario y pipeline; detalle y recuperación explícita.
Se conservó el checkout previo. Baseline/fuentes/backup/informes privados:
`~/.local/share/noteka-verification/2026-10-24-people-reference`.

Identidades acreditadas: once contactos mediante CSV/manifiesto/página original y
seis miembros mediante el otro CSV People. Once metadata de contacto, seis
procedencias de miembro, nueve fotos originales y 38 vínculos nuevos Project→Person.
Cero identidades omitidas. Las fotos corresponden a las imágenes servidas por
la galería observada; no se acredita resolución máxima del archivo remoto.
Backup canónico anterior a la primera escritura,
revisiones esperadas e informe incremental con payloads previos. Comparación final:
1.802 IDs y digest documental idénticos. No se enviaron documentos en updates ni
se escribieron páginas remotas.

La fuente autorizada se consultó por metadata de vistas y filas; los campos
ajenos a People no se copiaron al payload. El recorte se redondea al porcentaje
entero. Nueve de las primeras diez tarjetas tienen foto; la restante conserva
espacio neutro. El undécimo contacto se conserva sin inventarle una foto.

## Compilación e inspección

CLI: `cargo build --locked --offline -p noteka-cli`, aprobado. Contratos
regenerados y `npm run build` aprobado, cero errores/advertencias Svelte.
Persiste la advertencia previa Vite por bundle mayor a 500 kB.
Bundle final: `index-CT63rJXe.js`, 754,05 kB sin gzip; catálogo de 75 SVG estáticos,
con las ocho definiciones laterales conservadas.

Inspección manual asistida de UI de producción en Chromium headless, puente
de sólo lectura a CLI Rust y copia aislada del workspace recuperado:

- Contacts inicial: diez tarjetas, nueve fotos, orden/nombres y siete pestañas
  iguales a la referencia. Ancho de tarjeta 193,14 px a viewport 1920; primer
  renglón 226,19 px de alto. Load more: once contactos únicos, 11 de 11.
- New/Updated/Check-Ins: 0; las fechas originales no se sustituyen por la fecha
  de recuperación. Birthdays: una ocurrencia en octubre de 2026; navegación a
  noviembre y Hoy vuelve a octubre.
- Pipeline: once Prospect y las otras cuatro columnas vacías visibles.
- Active Projects: cuatro proyectos únicos y ocho apariciones en seis grupos
  de personas. Filas compactas con referencias locales y progreso real.
- Claro, oscuro y viewport 420 px: cero overflow global, cero errores de página
  y cero solicitudes externas. Galería alinea las fotos arriba; calendario y
  pipeline contienen su desplazamiento horizontal y ofrecen foco de teclado.

Se revisaron filtros previos a paginación, reglas civiles, límites, validadores,
guardias/callbacks y conservación de propiedades en el detalle. Estas inspecciones
no sustituyen suites de regresión ni prueban nuevas escrituras por UI. No se
agregaron ni ejecutaron suites; no hay nueva aceptación de IPC/cierre Tauri.

La app nativa se recompiló por el watcher de Tauri durante el cambio; los resultados
visuales citados corresponden a la copia headless. No se acredita sincronización continua,
fórmulas completas de Notion, recordatorios o arrastre de Pipeline.

Logs/capturas ignorados: `people-cargo-final.log`, `people-build-final.log`,
`people-recovery-{dry-run,apply}.log`, `people-preview-final.log` y
`people-reference-{light,dark,narrow,birthdays,pipeline,active-projects}.png`.
Los intentos iniciales fallidos de captura/validación se conservaron.
La app recompilada abrió Home y se pulsaron Tasks/Home mediante accesibilidad;
esto confirma controles disponibles, sin atribuirle prueba visual de la animación.

## OpenSpec

Cuatro tareas. Tres requisitos nuevos y dos modificados de desktop-shell y uno
nuevo de knowledge-entities. Se sincroniza únicamente el alcance completado tras
contrastar código, artefactos y evidencia. El resto de specs conserva su contenido.

Cierre: 4/4 tareas y archivo `2026-10-24-match-home-people-reference`.
Sincronización manual contrastada: desktop-shell 38→41 requisitos (dos previos
modificados explícitamente y los otros 36 preservados íntegros); knowledge-entities
11→12 con los once previos íntegros. Las otras quince specs conservaron sus hashes.
Validación estricta posterior al archivo: diecisiete specs, cero fallos. Esta
validación es documental y no sustituye la compilación/inspección registradas.
