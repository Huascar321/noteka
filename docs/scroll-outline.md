# Indicador persistente de secciones

Home y el detalle de una Note normal muestran un rail de marcas junto al borde
derecho del contenido. Permanece visible sin hover y resalta el tramo actual del
scroll local.

Ajuste visual del 26/10: marcas de 18/16/14/12px según nivel, grosor de 2px y
separación de 14px con mouse/trackpad. El rail mide 24px y la activa no usa brillo.
En Home, el contenido reserva 36 px en el borde final para conservar separación
con el rail y aprovechar más ancho junto al Side peek. En un panel de Home de
hasta 300 px, títulos y acciones se adaptan al ancho disponible.
Las áreas táctiles mantienen 24px.

Home representa **Tasks, Notes, Projects, Tags, Goals y People**. Note representa
el título de página y los encabezados H1/H2/H3 del documento. Los nombres
duplicados conservan marcas independientes; los encabezados de herramientas y
vistas incrustadas no forman parte del esquema. Una nota vacía o de formato no
editable conserva una marca de título sin transformar sus bloques.

Cada marca tiene nombre accesible y tooltip. Clic o Enter salta al título; las
flechas arriba/abajo y Home/End recorren las marcas por teclado. El salto respeta
movimiento reducido. El rail tiene scroll interno cuando hay muchos títulos y
reserva espacio para el contenido y el scrollbar.

Los cambios del documento, la paginación y el tamaño actualizan las posiciones;
las pestañas ocultas liberan observadores y listeners. Side peek y su origen
tienen indicadores independientes. La acción sólo cambia scroll: no guarda,
descarta, consulta SQLite ni modifica IDs/documentos/selección.

Implementación: ScrollOutline con caché de nodos/posiciones, observadores de
tamaño y contenido, RAF y búsqueda por posición durante el scroll, sin polling.
Ver [evidencia y límites](verification-scroll-outline.md) y [peeks](detail-peek.md).
