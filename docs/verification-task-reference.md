# Revisión de Tasks y arte de referencia

Fecha:23/10/2026. Cambio:match-home-task-reference.

## Evidencia de esta entrega

- Inspección de Notion y capturas provistas; recursos descargados sólo desde
  imágenes observadas en la sesión autorizada. La sesión permanece abierta.
- Baseline externo de17 archivos antes de editar, con hashes y cambios locales
  previos conservados. Sólo tres archivos Rust del núcleo se modificaron.
- Contratos generados, Svelte/TypeScript y build aprobados sin errores ni
  advertencias de Svelte; Vite mantiene su advertencia de bundle>500kB.
- cargo check y build de noteka-core/noteka-cli aprobados, offline/locked.
- Revisión visual headless del bundle index-i25fi-ST.js en claro/oscuro y
  viewport760px: sección712px, documento760px; no desbordamiento horizontal
  observado. Título32px, cuatro grupos,18 apariciones de imágenes para14 tareas.
- Se recorrieron también Today, Inbox, Week, Month, Scheduled, No Due y Active
  Projects sobre ese snapshot sin errores visibles; sus conteos fueron
  14/0/0/1/14/40/14 respectivamente, sin afirmar cobertura de casos límite.
- El recorrido observado reportó cero errores del navegador. No es una suite
  automatizada de aceptación ni acredita transporte/cierre de ventana nativa.
- Revisión de código de filtros previos a la página, identidad del cursor,
  límites, invalidación de imágenes y fallback. No se ejecutaron suites de
  tests ni se repitieron los gates históricos de límites/concurrencia.

## Preservación del workspace

Backup canónico antes de iconos y otro antes de vínculos. Cuatro adjuntos nuevos
para cinco proyectos;14 relaciones confirmadas por fuente explícita y CAS.
Un registro ambiguo se omitió primero y luego se resolvió mediante el ID
original del documento, Created/Edited/Status/Due/Priority y la correspondencia
completa de578 filas con el manifiesto del importador (incluido el recorte
de títulos a240 caracteres). Se conservan ambos informes y la prueba externa. No se crearon entidades:1802 IDs antes/después.
El hash SHA256 de todas las filas de documents ordenadas es idéntico:
`1f7a61d6a97da48823dc2977b4cd7941f54d675cedafda832994ecdb12aecf3c`.
Los14 cambios de tarea afectan revisión/updated_at; los cinco de proyecto además
su payload de icono. Estados, fechas civiles, títulos y documentos se conservan.
147 adjuntos previos permanecen; el total pasa a151. Los informes privados,
fuentes/capturas/backups están fuera de Git o en output/playwright ignorado.

## Límites

No se acredita copia de todos los filtros internos de Notion. Las vistas nuevas
usan las reglas locales documentadas; el modelo de tarea sigue siendo todo/done.
Recurring, Active Content, automatizaciones y filtros IA no se simulan. El
registro duplicado se asoció sólo después de resolver su identidad con la fuente. La inspección visual usa un snapshot
restaurado del workspace y un puente de transporte al CLI, no una sesión Tauri
instrumentada. Las suites de entregas anteriores no se presentan como prueba de
estos cambios nuevos.

OpenSpec:7/7 tareas cerradas; cuatro requisitos añadidos a desktop-shell.
Sus21 requisitos previos y las otras16 specs se conservaron. La validación
estricta documental aprobó; no sustituye la revisión funcional declarada arriba.
