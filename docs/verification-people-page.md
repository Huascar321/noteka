# Revisión: página lateral People

Fecha: 24 de octubre de 2026. Cambio `match-people-page-reference`.

## Compilación y preservación

`npm run build` completado: cero errores y cero advertencias Svelte. Bundle
`index-Cf-sXK_N.js`, 811,64 kB; CSS `index-UDNe0-b-.css`, 89,23 kB.
Persiste el aviso Vite existente de tamaño mayor de 500 kB.

Baseline privado de fuentes, guías y 17 specs; estilos anteriores conservados
íntegros como prefijo. HomePersonCard, fotos, HomeReader, contratos, Rust y datos
personales no cambiaron. PeoplePage agrega composición/lector; la variante Home
conserva su encabezado compacto y las mismas vistas. La opción informativa sobre
miembros del workspace ahora describe el acceso mediante búsqueda local.

La primera imagen mostró un círculo sólido por el relleno heredado del icono.
Se corrigió la presentación sólo en People lateral, reutilizando el SVG original
sin editar el catálogo. La nueva compilación y capturas incluyen la corrección.

## Inspección manual

Chromium headless → UI compilada → puente de operaciones sólo lectura → CLI real
→ copia restaurada canónica. La interfaz rechaza escrituras y solicitudes externas.
No se interactuó con la pantalla/IPC nativo ni se grabó audio.

- Home → People, icono/título/Nav/New Person y siete vistas observados. Contacts
  carga diez tarjetas y nueve fotos; Load more, pulsado en el extremo derecho de
  sus 1498 px, añade el undécimo contacto único y desaparece.
- New/Updated/Check-Ins vacíos en la fecha actual, sin errores. Birthdays presenta
  35 celdas del mes; Pipeline, cinco columnas y diez tarjetas en su primera página;
  Active Projects, seis grupos con proyectos originales.
- Cal abre octubre de 2026; avanzar a noviembre y actualizar conserva noviembre.
- Abrir una tarjeta conserva su ID/título y Home → People → nombre. Crear prepara
  título vacío y Home → People → Nueva persona; Guardar y continuar/Descartar/
  Cancelar protege la captura. Se descartó sin guardar.
- Claro/oscuro y 420 px inspeccionados. Documento 420/420 y sección 386/386;
  Pipeline desplaza 1028 px dentro de 386 px sin overflow global. Capturas tras
  estabilizar la disposición; menú y calendario siguen usando controles nativos.
- Inicio observado sin errores JavaScript ni solicitudes externas. Evidencia y
  capturas en `output/playwright/` ignorado, backup/copia y baseline fuera de Git.

## Límites

No se agregaron ni ejecutaron suites, escrituras de aceptación, arrastre,
inyección de fallos, carreras ni mediciones de rendimiento. La compilación y el
recorrido sobre copia no acreditan un nuevo recorrido nativo ni Windows.
Meeting Notes permanece en su propuesta; esta entrega no incorpora audio/ASR.

## Cierre documental

Un requisito añadido a desktop-shell (55 → 56), conservando los 55 anteriores,
Purpose y las otras 16 specs. Validación estricta de las 17 specs y del cambio
completada; archivo `2026-10-24-match-people-page-reference`, 3/3 tareas.
Meeting Notes mantiene su planificación independiente sin sincronizar a main.
