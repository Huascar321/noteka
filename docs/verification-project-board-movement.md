# Revisión acotada — movimiento de Projects

Fecha: 23/10/2026. Cambio `enable-project-board-dragging`.

## Evidencia aceptada

- `npm run build`: contratos vigentes, svelte-check con cero errores/advertencias
  y bundle `index-cUHMwkhL.js` (709,81 kB). Persiste la advertencia previa de Vite
  sobre chunks mayores a 500 kB.
- Revisión manual con UI de producción en Chromium headless → transporte aislado
  → CLI Rust real → copia del workspace de 1.802 entidades. Board: 12/12 tarjetas.
  Arrastre dentro de Doing, paso a Planned y regreso mediante diálogo: tres apply
  confirmados; la posición se conserva tras recargar y reabrir esa copia.
- Movimiento sobre la misma posición: contador de apply permanece 3. Escape cierra
  el diálogo; el foco inicial está en Cancelar y vuelve al botón Mover. Sin alertas
  en la revisión aceptada. Captura `output/playwright/project-drag-light.png`.
- Revisión de código: payload completo conservado, validación de entidad/revisión,
  contraste de Board posterior al preview y apply con receipt. El núcleo conserva
  validación/transacción/hash existentes; no se modificó Rust ni su contrato.

## Límites y fallos conservados

No se agregaron ni ejecutaron suites. No se simuló concurrencia ni fallo de red en
esta revisión; su tratamiento se contrastó en código con el contrato existente.
La revisión no acredita IPC WebKit ni cierre nativo Tauri. La primera compilación
falló por el tipo nullable de HomePage y se corrigió; el primer guion de revisión
esperó una reapertura automática inexistente tras reload, aunque los movimientos
ya se habían guardado. Ambos logs se conservan junto a los aceptados.

Los movimientos se hicieron únicamente en la copia temporal confinada del puente.
Los datos personales, capturas y logs no se incorporan a fuentes versionadas.
