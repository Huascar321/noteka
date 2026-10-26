# Revisión de herramientas y detalle Project

Fecha: 24/10/2026. Cambio `refine-page-tools-and-project-detail`.

## Inventario y referencia

Se encontró una sola implementación de Más opciones en WorkspacePane, compartida
por todos los detalles; las otras menciones eran documentación de ese control.
El [inventario por contexto](page-tools.md) cubre tipos, borradores y formatos
incompatibles. Las seis funciones de exportación/recursos conservaron su texto
exacto y sus condiciones; se cambió la envoltura de los controles.

El Project de referencia se identificó desde el vínculo de la Note ya usada:
se consultó la relación y la búsqueda de Notion, se abrió la coincidencia exacta
y se tomó una captura real con Playwright. Su ID de origen coincide con el del
Project local leído. No se editaron, desbloquearon ni recuperaron datos de Notion.
Capturas, IDs y cuerpos personales permanecen ignorados y en evidencia privada.

La captura muestra portada, icono/título, Status/Target Deadline/Progress,
relaciones y Tasks/Notes. La implementación usa portada decorativa local,
icono administrado, campos existentes y resumen calculado por Rust. No descarga
la portada ni copia un porcentaje fijo de la fuente.

## Compilación y conservación

- Backup canónico y baseline privado anteriores al cambio.
- `npm run build` aprobado, Svelte con 0 errores/0 advertencias. Se retiró una
  declaración de role redundante detectada en la primera compilación. El aviso
  previo de Vite por bundle mayor a 500 kB permanece.
- Bundle aceptado `index-D06fWRTi.js` y CSS `index-DwO4yUGy.css`.
- Rust, contratos, editor/adaptador, proyección importada, controles Note/Task y
  locks protegidos conservaron SHA-256. CSS anterior preservado como prefijo.
  Sin nueva dependencia, operación, documento o migración.
- Lectura canónica posterior: el Project completo, payload/documento y revisión
  7 coinciden exactamente con la lectura previa. No se invocaron mutaciones.

## Recorrido de interfaz sobre copia

Chromium → UI compilada → puente CLI de lectura → copia restaurada. El puente
bloquea escrituras antes de enviarlas; todos los borradores se descartaron.

- Project: estado/plazo editables en borrador, progreso Rust con décimas y
  completas/total. Cargar más acumuló los registros únicos restantes y retiró
  el cursor; Tasks agrupadas y Note original accesible. Las relaciones compactas
  no repiten todas las Tasks/Notes de las secciones.
- Cancelar la navegación después de editar conservó estado/plazo pendientes y
  captura deshabilitada. Nueva Note desde Project preparó el ContextLink correcto,
  sin crear entidad; sus grupos Exportar/Recursos permanecieron deshabilitados.
- Fallos de resumen/contexto y resumen ausente se simularon sólo en respuestas del
  navegador: error y reintento explícitos, No disponible distinto de cero. Al
  restituir las lecturas reaparecieron el resumen/cursor reales.
- Project presenta sólo Recursos externos. Enter/Espacio abre el panel; Tab
  llega a Cerrar y continúa a sus controles. Escape devuelve foco al disparador,
  expanded=false y tooltip visible en hover. Abrir/cerrar no generó llamadas.
- Note guardada: Exportar/Recursos externos/Adjuntos disponibles y cada aria-controls
  apunta a su panel real. Ruta de exportación y campo/DOM del preparador de adjuntos
  sobrevivieron a cerrar/reabrir. Tab con panel cerrado omitió controles ocultos.
- JSON/Markdown conservaron la condición de destino. Editar Type deshabilitó
  Exportar y la vinculación, manteniendo accesible el panel de recursos. Task sin
  documento mostró Exportar deshabilitado y Recursos habilitado, sin Adjuntos.
- Formato de Note desconocido simulado únicamente en el DTO leído: sin editor,
  JSON disponible y Markdown deshabilitado; se restituyó la lectura original.
- Con carga inicial terminada, abrir/cerrar los tres grupos de Note no generó
  llamadas. Cero intentos de exportar, vincular o procesar en todo el recorrido.
- El tooltip de grupos deshabilitados conserva opacidad completa; sólo el icono
  se atenúa. Se verificó el ajuste final de CSS en una carga nueva.
- Temas claro/oscuro y 420 px revisados. Project: viewport/documento/página
  420/420/420. Panel de herramientas Note: 380 px, sin overflow global.
  Capturas finales de ambos contextos guardadas; consola Noteka sin mensajes.

Las primeras consultas de controles de Note/Task se hicieron antes de finalizar
su apertura y registraron listas vacías y una lectura external.list tardía. La
revisión final espera la disponibilidad real y confirma los grupos y cero llamadas
por apertura. Dos intentos iniciales del controlador de búsqueda Notion usaron
un ref obsoleto y un filtro de texto inadecuado; se corrigieron con el nombre
accesible exacto y se preservaron los registros. No son fallos del producto.

## Límites y cierre

No se agregaron ni ejecutaron suites. Las comprobaciones UI fueron manuales y las
simulaciones sólo alteraron DTOs/error de la copia de revisión; no acreditan
persistencia, CAS real ni error real de almacenamiento. No se ejecutaron
exportaciones, Picker, llamadas a proveedores, extracción ni IPC/cierre nativos.
No se capturó ni cerró la app nativa. Eco Armario y voicebot quedaron fuera del
alcance y no se operaron sus repositorios ni procesos.

Tres requisitos añadidos sincronizados/contrastados: desktop-shell 60→63,
con todos los bytes de los sesenta requisitos previos conservados. Las otras
dieciséis specs permanecen intactas. Archivado 4/4 como
`2026-10-24-refine-page-tools-and-project-detail`; validación estricta final 18/18.
Meeting Notes conserva sus ocho artefactos y 0/19 tareas.
