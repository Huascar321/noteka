# Revisión PDF OCR

Revisión directa del20/10: código, diff contra checkpoint15, diseño, requisitos,
pruebas y gates. Sin defectos abiertos del recorte verificado. No se declara
fase5a completa. [Aceptación y límites](verification-pdf-ocr.md).

| Capability | Requisitos delta / escenarios | Evidencia |
| --- | --- | --- |
| core-operations |5/17|API55,52 operaciones comunes exactas, DTOs/CLI/SDK, batch9|
| local-mcp |7/19|27/47/48 herramientas, modern/legacy/auto y permisos, sin helpers|
| local-extraction |4/9|Raster/Vision reales, canónico/cola/lector,32MiB y UI450|
| local-text-search |2/4|Unicode/rangos512/513, corrupción fuera de filtro, cinco gates|
| local-workspace |2/6|Cliente/fixture15 previos,016/rollback/backup/unknowns|

La revisión conserva35 nombres de escenarios heredados de los once requisitos
modificados. Nueve requisitos nuevos; ocho capabilities no afectadas idénticas.
La propuesta de merge se validó estrictamente en copia externa antes de tocar
main specs. El lector independiente PDFKit y el OCR nativo anterior no cambian;
la única adaptación del supervisor Vision añade deny-network a PDF OCR sin
alterar la política del camino image_ocr existente. Los dos tipos de bbox
nombran el mismo raster orientation1, sin tratarlos como coordenadas originales.

Persistencia se valida entera antes de materializar/rendir páginas. Fuente y
canónico están vinculados por SHA, formatos/owners cruzados se rechazan,
particiones no canónicas fallan y el writer retiene los seis formatos. Proyección
indexa sólo texto y agrega manifiesto en una rama nueva; hashes previos exactos.
No-op/CAS/invalidation/errores y cuotas siguen compartidos. El ajuste Box privado
no cambia JSON ni contratos; se repitió foco por perfil y CLI/SDK posteriores.

Home, App, navegación, temas y sus artefactos protegidos siguen idénticos.
Verificación headless no sustituye aceptación nativa. Originales/buffers/unknowns
se conservan; no hay activaciones remotas, modelos ni publicación. El siguiente
trabajo es5a tabular/PDF/XLSX, después5b→5c; fase6 no se inicia.
