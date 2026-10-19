# Spec Delta

## MODIFIED Requirements

### Requirement: Lector XLSX independiente y sin activación implícita

El sistema SHALL ofrecer un lector local aislado de XLSX Transitional mediante un protocolo privado acotado de bytes, sin rutas ni acceso al workspace. MUST separar ese helper de las operaciones del core: importar, abrir, leer, encolar, indexar, buscar o migrar no inicia parsing; sólo `extraction.run` explícito sobre un trabajo admitido lo ejecuta. Index/search usan derivados ya confirmados, no el contenedor XLSX.

#### Scenario: Lectura explícita offline
- **WHEN** se invoca el helper con un XLSX sintético admitido y red denegada
- **THEN** devuelve una representación determinista completa del subconjunto declarado, sin modificar archivos ni iniciar UI, modelos o conexiones

#### Scenario: Producto existente conservado
- **WHEN** se consulta el catálogo, abre un workspace actual o importa/encola/lee/indexa/busca XLSX sin ejecutar run
- **THEN** API v1 conserva 52 operaciones y el contrato de extraction.xlsx_cells; sólo index/search amplían sus enums/DTOs y descriptor documentados, los otros 50 contratos y UTF-8/PDF/Home mantienen su comportamiento y no se inicia ningún helper

### Requirement: XLSX excluido de indexación textual implícita

El sistema MUST excluir selección XLSX implícita al importar, abrir, migrar, extraer o leer. SHALL admitir put explícito de xlsx_cells v1 íntegro dentro de cuotas, mediante campos tipados sin interpretarlo como UTF-8/PDF. Extractores/configuraciones futuros se rechazan sin descartarlos. Lectores, representación, búsqueda de notas/UTF-8/PDF, validación completa de postings/copias/fuentes, caps y target query p95 <=300 ms sobre los corpus fijados se conservan.

#### Scenario: Intento de indexar XLSX
- **WHEN** se intenta indexar explícitamente una generación XLSX existente junto a un índice UTF-8/PDF válido
- **THEN** el caso íntegro y dentro de cuotas añade sólo su selección; un caso incompatible, corrupto o excesivo falla sin cambiar el índice, búsqueda u originales previos ni ejecutar helpers

#### Scenario: Regresión de búsqueda y contratos
- **WHEN** se repiten el corpus/gate fijo y la extracción/lectura UTF-8/PDF/XLSX después de habilitar búsqueda XLSX
- **THEN** pasan validaciones completas y target sin subir caps u omitir fuentes; sólo los enums/DTOs de index/search y el descriptor de capacidades evolucionan de forma documentada, sin cambiar los otros 50 contratos ni los tres lectores
