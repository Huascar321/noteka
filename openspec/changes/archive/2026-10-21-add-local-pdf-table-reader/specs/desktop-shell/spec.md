# Spec Delta

## ADDED Requirements

### Requirement: Lectura de tablas PDF con grid explícito

El lector existente SHALL ofrecer página/grid/encabezado elegidos sobre PDF OCR, una página de20 celdas completas y cobertura/manifiesto/exteriores visibles. MUST escapar contenido y conservar buffers/guardias, criterios descartables y cupo físico; abrir o buscar no proyecta, prepara ni selecciona automáticamente.

#### Scenario: Tabla por teclado en ambos temas
- **WHEN** se consulta y pagina tabla con huecos, texto hostil, varios textos por celda y exteriores en dark/light
- **THEN** muestra procedencia y cobertura diferenciadas, texto/regiones completos sin ejecutar HTML; buffer, originales y selección FTS permanecen iguales, sin requests externos

#### Scenario: Error edición y salida tardía
- **WHEN** se cambia grid, Rust rechaza ambigüedad/capacidad o llega respuesta después de salir
- **THEN** retira página/cursor, mantiene error útil sin retry/fallback y descarta tardíos sin liberar anticipadamente IPC ni sobrescribir edición
