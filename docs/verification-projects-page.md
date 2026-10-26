# Revisión: página Projects

Fecha: 24 de octubre de 2026. Cambio `match-projects-page-reference`.

## Compilación y preservación

- `cargo build --locked --offline -p noteka-cli`, completado. Hubo espera por el
  bloqueo de compilación compartido; no se detuvieron procesos del usuario.
- `npm run contracts:generate` y comparación estructural: al retirar únicamente
  projects_on_hold/projects_by_tag/projects_by_people y related_entity opcional,
  el schema coincide con el baseline anterior. No cambian operaciones/payloads.
- `npm run icons:generate`: 76 SVG estáticos; los 75 anteriores son idénticos.
  Las ocho definiciones laterales y las cuatro licencias permanecen idénticas.
- `npm run build`, cero errores y cero advertencias Svelte. Persiste el aviso
  Vite del bundle mayor de 500 kB. Bundle final `index-BhnN8VF3.js`, 776,03 kB;
  CSS `index-D13ZbLQc.css`, 82,39 kB.
- Sin migraciones, cambios de datos ni nuevas dependencias. No se agregaron ni
  ejecutaron suites. La variante standalone y showState opcional conservan las
  composiciones previas de Home/Tasks y el editor individual.

## Fuente de referencia

Se leyeron estructura de Projects, destinos By Tag/By People y configuraciones
de sus vistas mediante el conector Notion. No se leyeron cuerpos de registros,
editaron propiedades remotas ni recuperaron datos al workspace. El conector marca
botones desconocidos en Nav; la captura del usuario acredita New Project y la
composición visible. Las configuraciones acreditan On Hold y agrupaciones por
relaciones. La evidencia completa queda privada fuera de Git.

Deadlines conserva la lista local por fecha existente; By Tag/People usa listas
plegables locales dentro de Projects. No se declara paridad de calendarios,
fórmulas de actividad ni todas las vistas auxiliares de Notion.

## Inspección manual

UI compilada en Chromium headless → puente de sólo lectura → CLI Rust real →
copia del workspace obtenida mediante backup/restauración canónicos fuera de
Git. El cliente bloquea mutaciones y solicitudes externas. No se abrió ni capturó
la pantalla nativa del usuario.

Observaciones del snapshot, no cantidades fijas de producto:

- Active inicia con siete filas, Doing 3/Ongoing 4, catorce imágenes locales,
  contexto, cantidades y progreso reales; breadcrumb Home → Projects, seis
  pestañas y controles de creación. El orden conserva board_order local y puede
  diferir del snapshot de Notion sin cambiar registros para igualar la captura.
- By Tag: doce proyectos únicos, once grupos, diez con identidad y Sin tag 1.
  By People: doce proyectos únicos en veintitrés apariciones, nueve grupos,
  ocho con identidad y Sin persona 3. Ambas incluyen Doing/Ongoing/Done/On Hold
  disponibles en este snapshot; no inventan proyectos para estados ausentes.
- On Hold muestra cuatro proyectos en un grupo on_hold, icono Pause real y
  New page de 1498 px, igual al ancho de sección.
- Activar New page desde el extremo derecho en On Hold abrió un borrador con
  estado on_hold y breadcrumb Home → Projects → Nuevo proyecto. Volver al
  antecesor mostró Guardar/Descartar/Cancelar; Descartar volvió sin guardar.
- Claro/oscuro y 420 px inspeccionados. Página/documento mantuvieron ancho y
  scrollWidth de 420 px; pestañas de 586 px se desplazan dentro de 386 px,
  y las dos filas New page miden 386 px. Metadatos pasan a otras líneas.
- La primera captura inmediata al redimensionar mostró una composición
  transitoria; se volvió a capturar tras esperar layout. El DOM tiene una sola
  barra de pestañas y un solo breadcrumb. No se cambió código por esa captura.
- Inicio final sin errores JavaScript ni solicitudes externas. El ajuste final
  de código conserva el retorno de foco a Board cuando ya no existe el control
  anterior del diálogo de movimiento; se revisó y compiló, sin guardar movimientos.
- Consola final: cero mensajes, errores y advertencias.

Capturas/logs en `output/playwright/` ignorado y evidencia privada. Las
observaciones de agrupaciones/captura anteriores al ajuste de foco mantienen
su alcance; las capturas finales amplias/estrechas usan el bundle final.

## Límites

No se ejecutaron suites, escrituras de proyectos/relaciones, movimiento de Board,
inyección de fallos, carreras, corpus con más de cien proyectos ni aceptación
nueva de teclado/IPC/cierre Tauri. Filtros de metadata inválida, cursores, caps,
descarte tardío, captura por relación y atajos se revisaron contra código y
contratos; esta inspección no vuelve a acreditar todas sus combinaciones.

## Cierre documental

Baseline privado de 38 archivos, hashes y comparaciones estructurales. Dos
requisitos desktop-shell se sincronizan únicamente después de esta revisión,
preservando los 49 bloques anteriores, Purpose y las otras 16 specs.
