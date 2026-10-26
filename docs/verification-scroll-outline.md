# Verificación del indicador persistente

Fecha: 26/10/2026. Cambio: `add-persistent-scroll-outline`. Estado: implementado y
verificado localmente; deltas pendientes de sincronización y archivo.

## Method

App compilada, Chromium headless aislado y transporte al CLI Rust/SQLite sobre
corpus sintético. No se inspeccionó pantalla nativa ni workspace personal.

## Results

Compilación aprobada, iconos/contratos vigentes y cero errores/advertencias
Svelte. **27 comprobaciones** focalizadas aprobaron en
`output/playwright/scroll-outline-20261026/`: seis marcas Home en orden, scroll y
clic de secciones, rail fijo, título + 26 headings H1/H2/H3, nombres duplicados,
salto local sin revisión ni estado dirty, fin del documento, scroll interno del
rail, pestañas aisladas/scroll conservado, renombre de heading e ID conservado,
teclado y ambos temas a 420px sin overflow, cero errores de navegador ni red
externa. La fixture de seis secciones incluyó áreas vacías reales.

La integración posterior de peeks aprobó indicadores simultáneos e independientes,
agregar/mover/eliminar un heading desde comandos de UI, borradores y una nota de
formato desconocido con sólo el título; el bloque desconocido conservó su JSON.
Estas comprobaciones están contadas dentro de las 100 de
[detail-peek](verification-detail-peek.md), sin sumarlas como otra suite.

## Compact refinement

Seguimiento visual del 26/10: compilación con cero errores/advertencias Svelte
y revisión focalizada headless de Home/Note, ambos temas y 420px. Medidas
observadas: rail 28px, marcas 22/19/16/13px según nivel, grosor 2px y paso 16px;
Home de seis marcas ocupa 100px incluyendo padding, frente a 150px previos.
Sin sombra activa; clic y flechas/Enter conservaron el destino local y no hubo
overflow global a 420px. Captura estrecha revisada y observaciones privadas en
`output/playwright/compact-outline-20261026/`. No se agregaron tests ni se
repitieron las suites anteriores para este cambio de presentación.

## Limits

Un segundo ajuste del 26/10, dentro de `resize-workspace-panels`, reduce el rail
a 24px y el paso a 14px, con marcas 18/16/14/12px y grosor de 2px. Se verificaron
medidas, teclado y ambos temas/420px dentro de las comprobaciones de
[paneles](verification-panel-widths.md); no se volvieron a contar las suites
anteriores. Para puntero grueso se conserva un objetivo de 24px.

La prueba comprueba el DOM compilado y persistencia local real mediante CLI,
sin acreditar IPC, scrollbar ni composición físicos de Tauri/macOS. No ejecutó
las suites históricas de toda la aplicación. El índice es visual/transitorio;
no genera un índice persistente, nuevos bloques, consultas ni migraciones.
Ver [guía](scroll-outline.md).
