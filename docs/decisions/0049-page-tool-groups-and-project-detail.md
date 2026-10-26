# ADR0049 — Paneles de herramientas y composición de Projects

Fecha: 24/10/2026. Estado: adoptada en la interfaz.

El control Más opciones era único y compartido por los detalles. Se reemplaza
por grupos funcionales con botones nativos y paneles inline, con snippets y
montaje conservado. Así cerrar un grupo no pierde el estado de un lector ni
cancela operaciones iniciadas. Se conserva la disponibilidad de las subacciones,
con nombres, tooltips, expanded/controls y retorno de foco al cerrar.

Projects obtiene una composición propia a partir de la referencia Notion
acreditada. Estado y plazo permanecen en el payload; progreso procede de Rust
por home.query/projects_board y las secciones de workflow.query/context. Un
cursor común acumula registros y comunica cobertura parcial. No se almacenan
porcentajes ni se reconstruyen reglas del núcleo en Svelte.

Se descartan un icono por operación, desmontar lectores al cerrar, cargar toda
la biblioteca para cada progreso, copiar porcentajes de la fuente y añadir
rich documents de Project sólo por apariencia. La portada es decorativa local,
con el icono original administrado. No hay migración, contrato o dependencia nueva.

La compilación/revisión sobre copia es evidencia de interfaz; no sustituye
persistencia, proveedores reales o IPC/cierre nativos. El alcance de las suites
y verificaciones anteriores se conserva por separado.
