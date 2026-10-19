# Tasks

## 1. Independent implementation

- [x] 1.1 Verificar baseline durable/contratos y desarrollar coordinador/lector de notas con AttachmentReader existente; aprobar tests de consulta exacta, páginas independientes, errores parciales, cambio de criterios, cupos/lifetime/remount y documentar alcance. Trazabilidad: consulta conjunta, páginas y aislamiento.
- [x] 1.2 Implementar panel inicialmente sin importar y luego integrado en App: grupos, fuentes de notas, evidencia/localizadores/cobertura completos de adjuntos y errores/controles accesibles; verificar check y presentación headless con golden real, texto hostil y ambos temas. Trazabilidad: evidencia y presentación.

## 2. Integration

- [x] 2.1 Tras aceptar los componentes previos, añadir tercera opción de Search y conectar apertura/refresh con App sin alterar ámbitos individuales, Home ni guardias; verificar build/check y montaje/cambio sin preparación. Trazabilidad: búsqueda de escritorio offline.
- [x] 2.2 Verificar headless integrado: paridad de ambos grupos con CLI, paginación independiente, error parcial/índice concurrente, respuestas tardías y buffer/apertura con guardia/teclado/viewport; conservar snapshots y registros sin writes o requests remotos. Trazabilidad: siete escenarios nuevos y tres heredados.

## 3. Integrated verification

- [x] 3.1 Comparar core/contratos/binarios/corpus/gates con aceptación reciente y documentar reutilización sólo si permanecen iguales; ejecutar regresión frontend/puente/headless de ámbitos individuales y lector pertinentes, y repetir verificación afectada ante diferencias o dudas. Trazabilidad: conservación/cuotas.
- [x] 3.2 Revisar trazabilidad de cuatro requisitos/diez escenarios, diff acotado, guías/plan/README/config y OpenSpec estricto, preservando los diecinueve requisitos ajenos y escenarios heredados. Trazabilidad: delta completo.

## Workflow follow-up

- Sincronizar sólo el delta implementado/verificado al completar seis tareas y archivar este cambio; conservar fuentes/evidencia y revalidar cierre/enlaces.
- Home sigue activo 9/17. No iniciar embeddings/agentes ni declarar 5a completa mientras falten sus otros recortes.
