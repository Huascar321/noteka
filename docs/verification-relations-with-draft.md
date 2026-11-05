# Verificación: relaciones con borrador pendiente

El 27/10/2026, `npm run build` pasó con cero errores o advertencias de Svelte; Vite mantiene el aviso previo de chunk mayor a 500 kB. El recorrido `node scripts/headless-relation-picker-smoke.mjs` pasó 33 verificaciones con UI compilada y workspace sintético.

El recorrido comprobó con operaciones reales del núcleo que agregar y quitar vínculos no guarda el título ni el cuerpo pendientes; el guardado posterior conserva tanto el contenido como la relación, y descartar el borrador no revierte una relación ya confirmada. También cubrió desvinculación entrante con borrador, página bloqueada, conflicto de revisión causado por una actualización externa real, y que el guardado posterior no sobrescriba esa actualización.

El transporte de prueba retrasó lecturas de entidad y de relaciones, además de una escritura: una respuesta obsoleta no produce un conflicto falso ni vuelve a mostrar un vínculo eliminado, y ⌘S no inicia otra escritura simultánea. Se inyectó un fallo de refresco posterior a una relación confirmada y se verificó el mensaje y la persistencia real del vínculo. El resto del recorrido conserva las pruebas de búsqueda, paginación, teclado y temas. No se validó IPC ni interacción nativa en macOS, Windows o Linux.

Las relaciones requieren una página ya creada: un borrador nuevo sin identidad debe guardarse una primera vez. Cada relación confirmada persiste por separado aunque luego se descarte el borrador de la página.
