# Revisión funcional PARA frente a Notion

Estado: **en curso**. El 05/11/2026 se consultó en modo de lectura el Home y
páginas de la referencia autorizada, sin modificar registros ni importar su
contenido. Los datos personales y las capturas del workspace remoto no forman
parte de este repositorio. La comparación de filtros no demuestra que todas las
fórmulas, automatizaciones o relaciones remotas sean equivalentes.

## Reglas observadas

- Tasks: Today de la referencia usa Status distinto de Done y Due hasta hoy;
  Scheduled usa Status distinto de Done y Due no vacío; No Due usa Status
  distinto de Done y Due vacío; All usa Status distinto de Done y Done usa
  Status Done. El Inbox remoto usa ausencia de Project y Content. Noteka tiene
  una decisión local previa para Today. El usuario confirmó conservar Inbox manual
  y sumar Unclassified por vínculos locales, sin comprobar Content. All se alineó con una consulta aditiva;
  [verificación](verification-pending-tasks-view.md). Week y Month se alinearon
  como calendarios navegables por Due con consultas por período;
  [verificación](verification-task-calendars.md).
- Notes: Fav. y las categorías de voz, diario y reuniones usan metadatos de
  tipo/favorito y excluyen archivo en origen. El Inbox remoto depende de la
  ausencia de Tag, Project, People y Content; el local mantiene la marca manual
  y agrega Unclassified según los tres vínculos locales, sin comprobar Content.
  Notes remoto admite Voice Note y exige URL vacía salvo ese tipo; la vista local
  ahora incluye Voice. El recuperador reconoce la etiqueta exportada Voice Note;
  no se ejecutó sobre datos reales. Daily y Journal se conservan como tipos
  distintos y aparecen juntos en Journal; Book, Recipe, Outlier Content, Creator
  Tips y Tools se admiten. Clips remoto también considera URL. La URL de origen local
  reside en el documento importado y no en la proyección Home. No se ha afirmado
  paridad ni replicado una regla sin información canónica disponible en la consulta.
- Projects: Active, Planned, Board, Completed y On Hold filtran estado/archivo
  como las vistas locales correspondientes. Deadlines se alineó como calendario
  mensual que incluye completados y sin fecha. El detalle remoto agrega Calendar
  mensual por Due y puede incluir tareas por Parent Project; el calendario local
  nuevo usa sólo relaciones efectivamente modeladas y avisa cuando la carga es
  parcial. La sección de notas del detalle remoto filtra por Project y excluye
  archivo, sin restringir el tipo; la sección local muestra notas relacionadas
  de todos los tipos. La ayuda remota describe avance como tareas completadas sobre tareas
  relacionadas, coherente con el cálculo local; los detalles de redondeo y
  exclusiones no se dan por equivalentes sin revisar la fórmula.
- Tags: Recent, Fav., A-Z, Types y Table usan exclusión de archivo; Fav. añade
  favorito. Table local es una presentación A-Z, no copia todas las propiedades
  configurables de la base remota. En el detalle, la sección Notes remota
  excluye Web Clip y contempla la relación Root Tag y una condición sobre URL;
  Noteka separa los clips, pero no acredita equivalencia para Root Tag ni URL.
- Goals: By Activity y By Deadline excluyen archivo; By Tag añade estado Active
  y Achieved añade estado Achieved. En By Tag, la referencia conserva las tres
  columnas Dream/Active/Achieved aunque sólo Active tenga elementos por el filtro;
  la presentación local mantiene esas columnas. Las fórmulas y ordenaciones no inspeccionadas
  no se consideran equivalentes por tener el mismo nombre. La ayuda del progreso
  remoto describe hitos completados sobre hitos vinculados, coherente con la
  proporción local; no acredita identidad de redondeos o manejo de errores.
- People: New/Updated usan un mes hacia atrás y Check-Ins hasta un mes hacia
  adelante; Active Projects requiere People y estados Doing/Ongoing sin archivo.
  La implementación local usa procedencia acreditada para fechas importadas y
  conserva esa decisión. Se añadieron campos locales de contacto e intereses sin
  copiar valores remotos; propiedades calculadas y reuniones no se replicaron.

Las pruebas y cambios concretos se documentan por separado. Recurrencias,
subtareas, Content, Meeting Notes y fórmulas remotas siguen sin acreditarse por
esta revisión. La equivalencia no es exhaustiva para todas las vistas, datos o
configuraciones de usuario. Ver [vistas Unclassified](verification-unclassified-views.md)
y [tipos de notas](verification-notion-note-types.md).
