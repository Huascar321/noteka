# Revisión de detalles Goal y Person

Fecha: 24/10/2026. Cambio `match-goal-and-person-detail-reference`.

## Referencias y alcance

Se buscaron páginas reales de Goal y Person en la misma sesión Notion usada
para Notes/Projects/Tags, se abrieron en página completa y se capturaron con
Playwright. Se inspeccionaron ambas capturas y las propiedades adicionales de
Person. Los source.record_id locales coinciden con las páginas observadas.
Notion no se editó; capturas e identidades personales quedan ignoradas y en
evidencia privada, fuera de Git.

Goal presenta Status/Tag/Progress, Nav, overview, tabla de hitos y proyectos.
Person presenta foto de portada, campos en View details y relaciones/vistas.
Los campos personales no soportados y Quick Notes/Gift Ideas de Notion no se
recuperaron. El usuario aclaró preservar acciones propias y el cuerpo existente
de Goal, sin ampliar texto libre a entidades core.

## Conservación

- Baseline privado de 53 archivos y backup consistente antes de editar.
  Antes de documentación, los 49 archivos fuera de los cuatro editados conservaron
  SHA-256: Rust, contratos, locks, PageTools, PersonPhoto, otros detalles, editor,
  diecisiete specs y ocho artefactos Meeting Notes.
- La invocación completa de BlockEditor y loadDraft conservan exactamente sus
  bytes respecto del baseline. No se cambió importador/adaptador, formato ni
  recomposición; no se retiraron bloques de plantilla del Goal.
- El CSS anterior permanece como prefijo completo. Sin Rust, contratos,
  migración, dependencia, nuevo cuerpo ni acción de dominio.
- Lecturas canónicas posteriores: Goal completo/revisión 7/payload/documento y
  Person completo/revisión 3/payload/documento idénticos a los previos.

## Compilación

`npm run build` final aprobado: iconos/contratos y Svelte cero errores y
advertencias; Vite conserva el aviso previo de bundle mayor a 500 kB.
Bundle aceptado `index-B5pylszX.js` y CSS `index-BZ84RQPO.css`.
La compilación inicial encontró nulabilidad del resumen y dos problemas de
tipado/orden de declaración; se corrigieron antes de la revisión UI. Los logs
iniciales y finales permanecen en la evidencia privada.

## Revisión de Goal

Chromium cargó UI compilada y CLI real sobre copia restaurada, con operaciones
de mutación bloqueadas antes del puente. El resumen observado fue 0/3 y 0%; tabla
de tres hitos con Target Deadline y Date Completed separados. El orden conserva
la consulta local, sin prometer el orden remoto de la captura.

- Guardado inicial, un editor existente y ninguna estructura de Note vacía.
  Goal Set y Target Deadline conservan sus fechas canónicas.
- Cambiar Status, plazo y contenido prepara Cambios pendientes; Exportar queda
  deshabilitado. Navegar muestra Guardar/Descartar/Cancelar. Cancelar conserva
  los tres cambios. El borrador se descartó sin guardar.
- New de Milestones prepara un hito con el Goal correcto y fecha completada
  vacía. Abrir un hito conserva su identidad/Goal; abrir un proyecto lleva al
  original guardado. Se descartó la captura de demostración.
- Fallos simulados de progreso e hitos muestran errores independientes y
  reintento. Resumen ausente muestra No disponible, resumen válido sin hitos
  muestra Sin hitos y estado desconocido se conserva Guardado.
- Conserva tres grupos de herramientas. Space abre Recursos externos y Escape
  cierra/devuelve foco. Abrir/cerrar no invocó operaciones adicionales.
- Los textos de plantilla importados permanecen en el cuerpo, incluidas sus
  etiquetas de Milestones/Projects/Nav. Son contenido conservado, distinto de
  las secciones operativas actuales. No se ocultaron para simular paridad.

## Revisión de Person

- Guardado inicial, foto local cargada con posición original y cero editores de
  cuerpo. El contexto real contiene un Project; no se copiaron los contadores
  mayores de Notes/Tasks de Notion.
- View details mantiene empresa, etiquetas, cumpleaños, check-ins y pipeline.
  Editar empresa/etiquetas/pipeline prepara el borrador; la captura se bloquea
  mientras está pendiente. Cancelar apertura del proyecto conserva los campos.
  El borrador se descartó.
- Captura Note y Task conserva ContextLink a la persona y Cambios pendientes;
  ambas se descartaron. Abrir el proyecto relacionado conserva su identidad.
- DTO de contexto con cursor y una página posterior duplicada deja un Project
  único y cursor agotado. La ausencia parcial se distingue de la completa.
  Fallo de contexto permite reintentar; foto inexistente deja portada neutra.
  Estas variantes se simularon sólo en el transporte del navegador, sin SQL.
- Único grupo aplicable Recursos externos: Space abre, Tab llega a Cerrar y
  Escape devuelve foco. Cero operaciones adicionales al abrir/cerrar.

## Presentación y límites

Capturas finales separadas para cada tipo, claras/oscuras y 420 px. Viewport y
documento 420/420; Person con propiedades abiertas mantiene 420. La tabla Goal
usa scroll horizontal interno. Sin alertas al terminar, consola local cero
mensajes/errores/advertencias y cero intentos de mutación. Carga inicial sin
solicitudes externas observadas. Ambos regresan a Guardado, sin texto de demo.

No se añadieron/ejecutaron suites ni se ejercitó guardado real, CAS, exportación,
proveedor, audio o IPC/cierre nativo. Los estados simulados acreditan frontend,
no reglas nuevas del núcleo ni persistencia. No se capturó/cerró Noteka nativo;
la previsualización propia se cerró y su servidor se detuvo. La sesión Notion
permanece abierta. Eco Armario y voicebot no se operaron.

## Cierre

Dos requisitos añadidos sincronizados y contrastados: desktop-shell 65→67.
Los 65 bloques previos, otras dieciséis specs y ocho artefactos Meeting Notes
conservan sus bytes. Archivado 4/4 como
`2026-10-24-match-goal-and-person-detail-reference`;
validación estricta final 18/18. Meeting Notes sigue propuesto 0/19.
