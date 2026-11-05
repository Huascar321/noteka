# Design

## Context

Ver `proposal.md` para motivación y alcance. `docs/design-system.md` es obligatorio según AGENTS.md y `src/style.css` contiene el material vigente. Existen siete verificaciones de esta serie y un puente headless que conecta el frontend compilado con CLI Rust/SQLite sintéticos. Las capturas temporales anteriores no son una galería durable.

## Goals / Non-Goals

**Goals:** permitir que otra IA reproduzca el diseño sin chats, con instrucciones aplicables al código actual y capturas trazables.

**Non-Goals:** nuevas decisiones visuales de producto, comparación pixel a pixel entre motores, nueva aceptación Tauri/macOS, cambios de datos o publicación de specs.

## Decisions

- Mantener una única guía principal y una galería enlazada. Registrar familias de tokens, valores e intensidades vigentes, con propiedad del CSS; evitar una segunda hoja de estilos en la documentación.
- Capturar el frontend de producción con Playwright CLI fijado y el puente existente, usando un workspace temporal, nombres ficticios e icono local como foto. No copiar screenshots de las notas/contactos reales adjuntos por el usuario.
- Guardar los PNG finales y un manifest saneado en `docs/design/liquid-glass/`; mantener artefactos operativos intermedios en `output/playwright/`, que Git ignora. El generador copia sólo los archivos aprobados a la galería al completar el recorrido.
- Incluir temas claro/oscuro, estados editable/bloqueado/pendiente, popups y fallback accesible; documentar el motor y los límites de blur. El contenido y la semántica se verifican antes de capturar.
- La fuente de verdad de datos sigue siendo Rust/SQLite. No hay migración ni nuevas operaciones; el script es una herramienta de desarrollo y el commit se hace sólo después de verificar los assets y el diff.

## Risks / Trade-offs

- [Capturas obsoletas] → comando de regeneración, inventario de archivos y huellas del código/PNG en manifest; actualizar junto al material.
- [Diferencias WebKit/Chromium y movimiento ASCII] → registrar motor/viewport/DPR/preferencias y reconocer que las imágenes son referencias de composición, no golden tests bit a bit.
- [Datos privados en Git] → corpus sintético nuevo y capturas sólo después de conectar ese corpus; no mostrar rutas de Workspace General ni pantallas nativas.
- [Historias de autorización contradictorias] → conservar evidencia histórica y actualizar el seguimiento con la solicitud actual de commit local, sin inferir push/sync/archivo.

## Migration Plan

No hay migración. Regenerar desde el build vigente, revisar las capturas y enlaces, validar documentos y diff, y realizar el commit local autorizado. El checkpoint `affca8a` conserva el estado anterior al experimento.
