# Spec Delta

## ADDED Requirements

### Requirement: Criterios explícitos de lectura de rango XLSX

El lector existente SHALL ofrecer hoja por ID, rango A1 y fila de encabezados opcional al elegir rango XLSX. MUST presentar cobertura de rango separada, encabezados/celdas/procedencia completos escapados y una página de20 items, con invalidación de criterios, errores útiles y descarte tardío sin alterar buffers/Home.

#### Scenario: Hoja rango y encabezados en ambos temas
- **WHEN** se consulta y pagina por teclado un rango XLSX con encabezados hostiles en dark/light
- **THEN** se muestran como datos las celdas tipadas, encabezados elegidos y manifiesto completo; abrir/editar criterios no consulta ni procesa y no hay requests externos

#### Scenario: Error cambio y salida
- **WHEN** se cambia criterio, falla la lectura o llega respuesta después de salir con nota sin guardar
- **THEN** se retira página/cursor sin retry automático, se conserva el buffer y se descarta respuesta tardía manteniendo el cupo físico hasta finalizar IPC
