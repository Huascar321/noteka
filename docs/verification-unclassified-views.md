# Verificación: vistas Unclassified

> Actualización 5/11/2026: la decisión de conservar Inbox manual fue sustituida. Inbox se deriva ahora de relaciones Project; Tasks ya no muestra Unclassified y Notes la conserva. Ver [ADR0059](decisions/0059-derived-inbox.md) y [verificación del Inbox derivado](verification-derived-inbox.md). El resto de este documento registra la verificación histórica de las vistas Unclassified.


El usuario eligió conservar Inbox manual y sumar vistas automáticas separadas.
Tasks incluye pendientes sin Project local; Notes, notas de biblioteca sin
Tag, Project ni People locales. Los vínculos a destinos archivados cuentan como
existentes. No se comprueba Content porque aún no está modelado. Abrir la vista
no cambia Inbox, entidades ni relaciones.

- Rust Home: 15 pruebas aprobadas. La nueva verifica Inbox independiente,
  estados, vínculos, destino archivado, quitar relación, paginación, cursor
  incompatible y rechazo de un vínculo con tipo incompatible.
- Chromium y WebKit: 12 checks por motor con CLI/SQLite sintéticos: ambas
  superficies, clasificación mediante los pickers reales, actualización de
  vistas, preservación de Inbox, aviso de Content y ancho de 420 px. Sin
  solicitudes externas ni errores de página.
- Build y contratos aprobados; Vite mantiene el aviso de chunk grande. La
  validación documental es independiente de la prueba de comportamiento.

Las vistas sólo reflejan relaciones locales guardadas, no datos remotos aún no
modelados. Las consultas son aditivas: clientes con enums cerrados necesitan
actualizarlos para solicitarlas. No se hizo aceptación nativa Tauri/macOS, ni
sync o archivo del delta.
