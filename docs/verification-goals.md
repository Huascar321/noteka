# Revisión: Goals lateral y Milestones

Fecha: 24 de octubre de 2026. Cambio `add-goals-sidebar-and-milestones`.

## Compilación y contrato

- `cargo build -p noteka-cli --locked --offline` completado con los subtipos,
  validaciones, referencias y proyecciones nuevos. `contracts:generate` y
  `contracts:check` completados.
- `npm run build`: cero errores y cero advertencias Svelte. Bundle
  `index-DEtMnc49.js`, 808,05 kB; CSS `index-BsvmQA0X.css`, 88,95 kB y módulo
  lateral diferido `sidebar-icon-animation-CLl0JT-u.js`, 76,34 kB. Persiste el
  aviso Vite existente del bundle mayor de 500 kB.
- Contrato anterior exactamente igual tras retirar sólo adiciones declaradas:
  cuatro definiciones Goal/Milestone, campos NoteFields/HomeMetadata/HomeQuery,
  seis HomeView y variante HomeSummary::Goal. Los 76 SVG estáticos permanecen
  idénticos; se agrega trophy existente a las nueve definiciones laterales.
- SQLite 20, mismos EntityKind y operaciones públicas, sin nueva dependencia ni
  migración. Un literal anterior de HomeQuery recibió goal_id:None y el caso
  existente de menú en context-search espera siete destinos; sólo adaptación
  de compatibilidad y revisión de sintaxis, sin casos nuevos ni suites ejecutadas.

## Revisión de código

Rust valida subtipos excluyentes, estados, fechas, snapshot/IDs y referencias a
Goal del mismo workspace, observa la dependencia en MutationContext, conserva
vínculos anteriores a archivados y rechaza asignaciones nuevas o retirar un Goal
referenciado. El tag directo es único. CRUD/CAS y rollback por transacción siguen
compartidos; lectura no expande documentos ni modifica progreso/estado.

Proyección Goals en transacción con cursores contextualizados y caps previos;
fuentes relacionadas se acotan por su conjunto acumulado, no por cada consulta.
Resumen con todos los hitos activos, ausencia frente a cero y estado independiente.
Actividad compara revisiones de recuperación con fuentes posteriores. El mapa de
revisiones persistido no se expande en filas Home. UI usa lectores con lifetime,
cola y descarte de obsoletos, variantes acotadas, atajos activos y guardias.

## Recuperación de datos reales

Fuentes: esquemas/filas y configuraciones conectadas de Goals/Milestones, más
lectura de fórmulas y valores visibles en la sesión Notion autorizada. List de
Milestones filtra Date Completed; By Tag sólo Active. El fetch de la página
marca controles Action Center no interpretados y un bloque no compartido; esos
elementos no se usan como evidencia de funciones implementadas.

Helper por IDs originales con planificación previa, backup canónico nuevo, CAS
e informe incremental privado. Tres Goals tipados, dieciséis Milestones, ocho
hitos vinculados y tres relaciones a tags agregadas. Cinco vínculos a proyectos
existentes conservados; la plantilla Goal permanece sin promover. Se añadieron
tres snapshots de actividad con precisión de minuto, zona America/La_Paz y
68/27/86 revisiones de fuentes. No se copiaron porcentajes ni cuerpos en updates.

Antes/después: 1.802 IDs idénticos y digest documental idéntico,
registrado en los manifiestos privados.
Backups y fuentes se conservan fuera de Git. La recuperación es incremental;
no hay reversión automática sobre cambios posteriores del usuario.

Lecturas CLI reales tras recuperar, no cantidades fijas del producto:

- Goals: tres Active, Dream/Achieved vacíos, resúmenes con 3/2/3 hitos,
  cero completados y progreso 0%, sin metadatos no interpretables.
- Actividad: fechas de la observación original conservadas, con precisión de
  minuto y zona explícita; la recuperación no se convirtió en actividad de hoy.
- By Deadline contiene los tres; By Tag, tres grupos. Achieved cero;
  MilestonesCompleted cero y MilestonesAll dieciséis. Plazo real disponible
  en un Goal; no se asignaron fechas faltantes para igualar la referencia.

## Inspección manual de UI

Chromium headless → UI compilada → puente local sólo lectura → CLI Rust real
→ copia restaurada canónica del workspace. Cliente rechaza operaciones mutantes
y solicitudes externas. Preparar/descartar captura sólo cambia memoria de UI;
la recuperación explícita anterior es el único cambio de datos de esta entrega.

- Menú observado: Home/Tasks/Notes/Projects/Tags/Goals/People. Página con trofeo,
  Nav, cuatro vistas y breadcrumb Home → Goals. By Activity presenta tres filas
  con tags originales, progreso y fecha; grupos Active3/Dream0/Achieved0.
- By Deadline ubica primero el Goal con plazo. By Tag muestra tres tags, tres
  tarjetas y nueve columnas de estado. Achieved está vacío con su grupo real.
- Detalle Goal abre el mismo título, estado Active, tres hitos y cuatro relaciones
  en el caso observado. Milestone muestra selector con los tres títulos de Goal,
  vínculo correcto, plazo y Date Completed vacío. Goal e hito conservan ruta
  Home → Goals → título. Opciones de Goal permiten actualizar su listado.
- Crear hito desde el Goal prepara título vacío y su vínculo. New page en Dream,
  activado desde el extremo derecho de la fila de 1498 px, prepara Goal vacío
  con Dream. Guardar y continuar/Descartar/Cancelar conserva la guardia; se
  descartaron ambas capturas sin guardar.
- Claro/oscuro y 420 px inspeccionados. Listas, columnas y detalle mantuvieron
  documento 420/420 px y sección 386/386 px; By Tag desplaza 654 px dentro de
  386 px. Los títulos permanecen accesibles y las columnas no desbordan el documento.
- Goals en Home muestra tres originales en posición entre Tags y People,
  sustituyendo el aviso pendiente y conservando el acceso a Goals lateral.
- La presentación de propiedades duplicadas conserva prefijo/IDs; los campos
  de origen quedan en View details y el contenido posterior sigue disponible.
  No se convierte el resto del template de Notion en vistas o automatizaciones.
- Inicio observado sin errores JavaScript ni solicitudes externas. Capturas/logs
  en `output/playwright/` ignorado y evidencia privada.

El primer intento de inspeccionar una opción nativa esperaba visibilidad con el
select cerrado; se corrigió leyendo sus opciones y el valor seleccionado. La
primera compilación Rust tuvo un préstamo/movimiento de etiqueta corregido antes
de la recuperación. Ambos logs se conservan. Una espera de animaciones del driver quedó pendiente después de
las capturas finales; se canceló al recargar sólo la vista previa y se completó
la observación de Home en una llamada acotada. No se atribuye ese bloqueo a la app.
La presentación duplicada se ajustó
tras la inspección, conservando sólo nombres conocidos tras el marcador canónico.

## Preservación y límites

Baseline y manifiestos privados de código/contratos/specs antes de editar; CSS
anterior conservado íntegro como prefijo. Se sincronizaron sólo seis requisitos
añadidos y tres modificados, conservando nombres de escenarios heredados,
Purpose, 72 requisitos anteriores no afectados y las otras 14 specs. Validación
estricta anterior al archivo: 18/18; posterior: 17/17, sin cambios activos.
Cambio cerrado 4/4 como `2026-10-24-add-goals-sidebar-and-milestones`.
OpenSpec no verifica el comportamiento de la aplicación.

No se agregaron ni ejecutaron suites, guardados de aceptación, inyección de
fallos, carreras, cuotas máximas o mediciones de rendimiento. No hubo nueva
aceptación de teclado/IPC/cierre nativo Tauri ni activación de modelos/proveedores.
La evidencia de integridad/CAS de los helpers y las guardias compartidas conserva
su alcance histórico; la revisión de código/compilación no equivale a reejecutarla.
