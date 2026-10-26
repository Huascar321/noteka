# Spec Delta

## ADDED Requirements

### Requirement: Selección PDF OCR y localizadores inferidos

El índice SHALL admitir pdf_ocr sólo mediante put con generación/revisión esperadas y proyectar texto de regiones en fragmentos<=4096 bytes UTF-8 sin cruzarlas. MUST conservar página, ordinal local/global, perfil/motor, raster/hash/dimensiones/rotación, bbox/confianza y rango derivado; no indexar JSON/metadatos ni cambiar hashes de los cinco formatos anteriores.

#### Scenario: Vacío Unicode y selección sin helper
- **WHEN** se selecciona un PDF OCR íntegro con regiones vacías o largas/Unicode y no hay helpers
- **THEN** sólo texto reconocido no vacío aporta fragmentos, cuyos rangos reconstruyen las regiones; conserva páginas sin regiones en manifiesto y permite selección vacía sin afirmar ausencia visual

#### Scenario: Integridad CAS y presupuestos
- **WHEN** hay corrupción fuera de filtro/página, fuente de candidato alterada, CAS obsoleto, reemplazo o exceso512 fragmentos/1MiB por selección,8MiB global o2MiB por página
- **THEN** falla completa o invalida selección/cursor conforme al núcleo; no omite validaciones ni purga/trunca fuentes, y mantiene query p95<=300ms/lock p95<=150ms

### Requirement: Evidencia PDF OCR en búsqueda de escritorio

Search individual/agrupada SHALL ofrecer pdf_ocr y presentar página/región/raster, texto/procedencia/localizador completos e inferencia falible, sin tratar bbox como offsets originales o confianza como certeza. MUST conservar filtros/grupos/cobertura/cursores/errores y contenido escapado, sin ejecutar extracción/indexación/lectura adicional ni modificar Home/buffers.

#### Scenario: Lectura contrastable y páginas independientes
- **WHEN** se buscan regiones seleccionadas y entidades, con paginación y contenido hostil
- **THEN** los hits coinciden con rangos del lector real, los grupos conservan identidad/cobertura y texto se presenta como datos sin mezclar scores o cursores

#### Scenario: Sin selección o error de un grupo
- **WHEN** existe extracción sin put o falla/vencen criterios de adjuntos durante paginación
- **THEN** la primera no aporta hits; el error retira sólo la evidencia afectada sin convertirlo en ausencia universal ni borrar el otro snapshot
