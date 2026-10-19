# Tasks

## 1. Independent reader

- [x] 1.1 Verificar baseline durable/contratos observados e implementar controlador explícito de tres lectores/20 items, identidad/generación/error y lifetime/cupo físico; aprobar tests de cliente para páginas, vacío, rechazo/stale, remount y cancelación en cola, y documentar los contratos. Trazabilidad: lectura, páginas y aislamiento.
- [x] 1.2 Implementar panel todavía sin importar en App, con texto/localizadores UTF-8/PDF y celdas/manifiestos XLSX fieles; aprobar check de tipos y tests de presentación/fixtures sin convertir lexemas ni omitir metadata. Documentar límites y diferencia respecto de original visual. Trazabilidad: presentación tipada.

## 2. Integration after preparation acceptance

- [x] 2.1 Integrar tras aceptación UI aislada y revalidar ambas UIs juntas; integrar panel/puente headless sin alterar Home/guardias y preparar fixture externa CLI con golden, más de veinte items, vacíos y todos los tipos/caches; verificar check/build y ausencia de consultas al montar. Trazabilidad: lectura explícita y conservación.
- [x] 2.2 Completar headless ambos temas con lectura/paginación exacta, fuente/generación concurrentes, error/ID/tipo inválidos, respuesta tardía/remount, buffers y teclado/viewport; comprobar snapshots sin writes/modelos/remotos y conservar logs/capturas/fallos. Trazabilidad: ocho escenarios completos.

## 3. Integrated verification

- [x] 3.1 Contrastar hashes de Rust/contratos/binarios/corpus/gates con aceptación reciente de preparación y documentar reutilización delimitada de sus 36 etapas/cuatro gates, sin declararlos nuevas ejecuciones; si difieren, repetir verificación afectada. Aprobar frontend/controladores/puente/headless previos pertinentes y build host sin ventana. Trazabilidad: preservación y cuotas.
- [x] 3.2 Revisar cuatro requisitos/ocho escenarios/diseño y diff contra baseline, actualizar plan/guías/README/config sólo con evidencia, preservar Home y validación estricta OpenSpec. Trazabilidad: delta completo.

## Workflow follow-up

- Sincronizar solamente el delta implementado/verificado cuando las seis tareas estén completas; preservar requisitos de preparación y Home, y archivar sólo este cambio.
- Conservar manifiestos/evidencia y revalidar cierre/enlaces; OCR y ampliaciones PDF/Excel continúan como recortes de 5a. La búsqueda agrupada se verifica en `add-grouped-local-search` por separado.
