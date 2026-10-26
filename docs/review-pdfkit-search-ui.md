# Revisión de PDFKit en búsqueda e interfaz

20/10/2026. Revisión directa del diff, diseño, requisitos y pruebas del cambio
`add-local-pdfkit-search-ui`. [Aceptación](verification-pdfkit-search-ui.md).
No quedan defectos abiertos del recorte verificado; la fase5a continúa incompleta.

| Capability | Requisitos delta / escenarios | Evidencia |
| --- | --- | --- |
| core-operations | 6 /20 | Catálogo14 real,52 operaciones exactas; SDK por perfil/era, permisos y DTOs; batch9/exclusiones7 intactos |
| local-extraction | 4 /8 | Seis tests Rust nuevos, corpus nativo CLI por perfil,822 checks UI; canónico/proyección/metadata/errores/cercas/buffer |
| local-text-search | 4 /15 | Selección explícita, corrupción fuera de filtro, oracle de rangos, hits nativos reales, Search20/3 y grupos independientes |
| local-workspace | 3 /10 | Fixture14 anterior a015,41 tablas/filas/originales/cursor/hash exactos, backup14/rollback/clientes14/16, unknowns y snapshots |

Se conservan39 nombres de escenarios heredados y115 requisitos no afectados de
las cuatro specs; nueve capabilities ajenas permanecen idénticas. La primera
copia de sincronización falló sólo por longitud>500 del statement; la segunda
conserva escenarios y valida estrictamente. No se confunde validación documental
con pruebas de aplicación. Las cláusulas antiguas de versiones se aclararon por
cliente para evitar llamar futuro al formato que ahora usa el producto.

El hash nativo incorpora manifiesto completo por una rama propia; los otros cuatro
formatos conservan hashes y shapes. SQL limita tamaños/owners antes de cargar;
la proyección valida y no indexa JSON/metadatos. Lectura/índice/búsqueda funcionan
sin helper. No-op/reemplazo/corrupción y source-candidate fences se verifican con
fuentes independientes y tests de conservación, no sólo snapshots de TypeScript.
El máximo nativo de fragmentación queda por debajo de512 por límites más fuertes;
se mantiene ese cap y se rechaza exceso entero. No se trunca para dar éxito.

UI añade sólo el caso PDFKit a los paneles/controlador existentes y al puente de
pruebas. Textos activos permanecen escapados; buffers/guardias/cupos siguen vigentes.
App/Home/temas y seis artefactos Home permanecen byte a byte iguales. No hay UI
nativa ejecutada ni aceptación de sus ocho tareas pendientes. Los fallos inyectados
están identificados; los errores nativos de PDF inválido se prueban por separado.

Pasaron345 Rust por perfil,25 SDK por perfil,86 cliente/editor y4264 checks
headless. Se conservan los intentos fallidos de harness/compile/validación con sus
correcciones concretas. No hubo fallo de rendimiento en este cambio: cuatro gates
originales y dos corpus adicionales pasaron300/150 con17 binarios fijados; carga
nativa grande3.519.600 bytes/400 páginas, query143,365ms/lock10,405ms. No se amplían
límites ni se retiran comprobaciones. No implica garantía universal de latencia.

La migración015 sólo es barrera de compatibilidad; el cliente14 preservado rechaza
el formato nuevo. Fuentes, buffers, propiedades/bloques/configuraciones desconocidos
se conservan. El lector independiente PDFKit archivado no se recrea ni modifica.
La comparación histórica exacta v11 perdida no se presenta como recuperada.

OCR dePDF, tablas PDF y hojas/encabezados/rangos XLSX siguen como alcance posterior
de5a.5b/5c no comenzaron. Home permanece9/17. No se activa proveedor/modelo/OAuth,
se instala peso, se publica o se implementa fase6. Sólo este recorte finalizado se
sincroniza/archiva; los pendientes no se publican en main specs como implementados.
