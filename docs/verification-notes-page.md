# Revisión: página Notes

Fecha: 24 de octubre de 2026. Cambio `match-notes-page-reference`.

## Alcance y compilación

- `npm run build` completado, con cero errores y cero advertencias Svelte.
  Contratos e iconos vigentes. Persiste el aviso Vite del bundle mayor de 500 kB.
- Bundle `index-7UlLg1gi.js`, 769,33 kB; CSS `index-BsA3XZwk.css`, 79,87 kB.
- Sin cambios Rust, contratos, SQLite, payloads, documentos o relaciones.
  No se agregaron ni ejecutaron suites ni se inició una aceptación nativa.
- NotesPage reutiliza lectores/cola y consultas existentes. Revisión de código:
  vistas independientes, invalidación/dispose, filtros en Rust, semillas de captura,
  callbacks con guardias y atajo acotado a página activa, sin recarga del WebView.
  Home conserva variantes por defecto; el editor individual no cambia.

## Inspección manual

UI compilada en Chromium headless → puente local de sólo lectura → CLI Rust real
→ copia del workspace mediante backup/restauración canónicos fuera de Git.
El cliente rechaza operaciones mutantes y solicitudes externas; crear sólo
prepara memoria del cliente. No se abrió la pantalla nativa del usuario.

Observaciones del snapshot, no cantidades fijas de producto:

- Notes inicia en Notes y Tags en Recent, con diez filas por sección. Breadcrumb
  Home → Notes, icono/título grande, Nav, New Note y ocho pestañas; Tags presenta
  sus cuatro vistas debajo. Las filas muestran arte/relaciones/tipos/antigüedad
  y última nota/clasificación disponibles desde originales locales.
- Load more activado desde el extremo derecho pasó de diez a veinte notas,
  con veinte IDs únicos. Sección, Load more y New page midieron 1498 px;
  New Note conservó 109,61 px y borde de 1 px.
- New page desde el extremo derecho abrió un borrador sin título, breadcrumb
  Home → Notes → Nueva nota. Navegar al antecesor mostró Guardar/Descartar/Cancelar;
  Descartar volvió a Notes sin guardar. El primer selector de observación del
  título no existía; se corrigió usando el nombre accesible real y se observó vacío.
- Claro/oscuro y 420 px inspeccionados. La página y documento mantuvieron
  clientWidth/scrollWidth de 420/420; las pestañas Notes usan scroll interno
  de 646 px en 386 px disponibles y las acciones textuales miden 386 px.
- El atajo Nav → Tags desplaza y enfoca su encabezado, manteniendo el breadcrumb.
- Clips mostró diez filas Web Clip; alternar Tags a Types mantuvo Clips y mostró
  los grupos Area/Resource/Entity con la cobertura de su página. Volver a
  Notes/Recent restableció sus consultas independientes.
- Sin errores JavaScript ni solicitudes externas en el inicio observado.
  Recursos de imagen locales; sin activar modelos/proveedores.
- Consola al terminar: cero mensajes, errores y advertencias.

Capturas y logs se conservan en `output/playwright/` ignorado y en evidencia
privada fuera de Git. Las capturas amplias incluyen ambas secciones; las estrechas
conservan el contenido mediante scroll vertical y pestañas con scroll propio.

## Límites

No se ejecutaron suites, guardados, inyección de fallos, carreras, todas las
combinaciones de metadatos ni aceptación nueva de teclado/IPC/cierre Tauri.
Los contratos de lectura/guardias compartidos mantienen su evidencia histórica;
compilar e inspeccionar esta UI no vuelve a acreditar esos recorridos completos.
Las fechas de la referencia pueden diferir del snapshot local por su antigüedad
y el día civil actual; no se actualizaron datos personales para igualar una captura.

## Preservación y cierre

Baseline privado de 27 archivos y manifiesto SHA-256 antes de editar, conservando
los cambios previos del checkout. Delta de dos requisitos desktop-shell; el cierre
contrasta los 47 requisitos previos y las otras 16 specs. Se sincroniza únicamente
la composición implementada y revisada, conservando Purpose y contenido previo.
