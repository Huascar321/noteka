# Verificación del selector de relaciones

Esta evidencia es histórica: la exigencia de guardar el borrador fue sustituida
por [edición independiente de relaciones](verification-relations-with-draft.md).

El 27/10/2026, `npm run build` pasó con cero errores y advertencias de Svelte; Vite conserva el aviso previo de chunk mayor a 500 kB. `node scripts/headless-relation-picker-smoke.mjs` pasó 23 verificaciones con UI compilada, CLI y workspace sintético: búsqueda por nombre y sin acentos, selección y eliminación, teclado y foco, guardado explícito y fallo de guardado del borrador, protección de página bloqueada, propiedad Project sin duplicación, paginación, búsqueda en una página posterior y respuesta obsoleta, error/reintento, conflicto simulado, desvinculación entrante y entidad archivada conservada.

La paginación tardía, errores y conflicto se inyectaron en el transporte de prueba; los enlaces y desvínculos restantes usaron operaciones reales del núcleo. Se revisaron capturas sintéticas claras y oscuras a 420 px y la posición del selector dentro de la ventana. No se probó IPC ni interacción nativa de macOS/Windows/Linux. Una primera prueba detectó pérdida de foco tras un cambio y se corrigió capturando el control antes de desmontar el selector; también se ajustó la sincronización del recorrido de prueba.

La búsqueda recorre páginas locales de 100 entidades, sin depender de FTS: para una consulta sin coincidencias en una página continúa hasta encontrar alguna o agotar la lista. Con resultados y páginas pendientes ofrece Load more. Un workspace grande puede requerir varias lecturas; el estado Searching y la cancelación al cambiar búsqueda o cerrar evitan presentar cobertura parcial como ausencia completa.
