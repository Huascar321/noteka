# Design

## Context

Ver proposal.md. App ya conserva paneles montados por pestaña y una autoridad de cierre. El lateral sólo se oculta mediante CSS en menos de 760 px; el botón actual no pliega escritorio. Tauri 2.12 ofrece Overlay en Mac y ventanas sin decoración para la barra propia de Windows.

## Goals / Non-Goals

**Goals:** integrar la fila superior sin tocar operaciones del núcleo y mantener estado/foco de la sesión.

**Non-Goals:** nuevas ventanas, reordenar pestañas, persistir sesiones, sincronización web, captura de pantalla nativa o implementar Meeting Notes. La comprobación nativa de Windows queda diferida por decisión explícita del usuario.

## Decisions

1. Un componente WindowTitlebar encima de la shell usa el mismo snippet de pestañas; los WorkspacePane permanecen montados bajo los mismos IDs. La bienvenida también conserva controles Windows disponibles.
2. Configuraciones Tauri específicas: macOS Overlay/hiddenTitle/decorations=true y posición explícita de controles dentro de una barra de 44 px; Windows decorations=false y controles HTML a la derecha. Los archivos por plataforma repiten geometría/label completos porque los arrays JSON de configuración se reemplazan. Linux conserva decoración normal. No se reemplazan los controles Mac con imitaciones.
3. La selección de presentación usa isTauri y el identificador de plataforma del WebView; el navegador siempre usa web aunque corra sobre Mac o Windows. No participa de autorización ni del acceso a datos. El componente sólo invoca APIs de ventana en el host nativo.
4. Zonas data-tauri-drag-region en espacios libres, sin la marca en botones. Permisos de minimizar/maximizar/consultar estado/arrastre limitados a main; no permisos amplios de shell/filesystem. Cerrar Windows llama a la misma autoridad leaveWorkspace(true), no directamente a app.exit desde un control independiente.
5. Estado de presentación separado: sidebarCollapsed para escritorio y sidebarOpen para panel estrecho, con matchMedia y retiro de listeners al desmontar. La navegación oculta es inert/aria-hidden. Cambiar ancho cierra el panel superpuesto y conserva la preferencia de escritorio durante la sesión.
6. Sin nueva persistencia ni migraciones. La lectura visual utiliza una copia restaurada y un puente de sólo lectura; variantes Mac/Windows en Chromium son simulaciones de composición, no prueba de controles nativos.

## Risks / Trade-offs

- Posición de botones Mac y fullscreen depende del sistema → reserva de ancho explícita y configuración admitida por Tauri; compilación macOS, aceptación visual nativa pendiente sin autorización de pantalla.
- Windows sin decoración requiere revisión en máquina real → controles accesibles, guardia compartida y pendiente documentado. No se promete Snap Layout al pasar el puntero sobre un botón HTML.
- Error al consultar estado maximizado → error visible y controles disponibles; listener y consultas no publican resultados tras desmontar.
- Cambio de configuración exige recrear ventana → registrar que recargar sólo el frontend no aplica la decoración nueva, sin cerrar la sesión nativa del usuario.

## Migration Plan

No hay migración de datos. Aplicar configuración al próximo inicio del host. Revertir los archivos de presentación/configuración restaura la barra previa sin tocar SQLite, adjuntos ni documentos. Mantener evidencia de compilación y el alcance real de la inspección.

## Sources

- https://v2.tauri.app/learn/window-customization/
- https://v2.tauri.app/reference/config/#windowconfig
- Schema y APIs instaladas @tauri-apps/cli 2.12.2 / api 2.12.3, consultadas el 24/10/2026.
