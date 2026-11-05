# Verificación del editor de bloques de Tasks

28/10/2026. `align-task-block-editor` activa en Task el editor contextual
compartido con Note y reconoce el espacio inferior de ambos detalles. No modifica
Rust, el formato de documentos ni el esquema SQLite.

`npm run build` completó con cero errores y advertencias de Svelte; Vite informó
un chunk superior a 500 kB. `node scripts/headless-task-editor-smoke.mjs` y su
variante `NOTEKA_TEST_BROWSER=webkit` aprobaron 18 comprobaciones cada uno con
UI compilada, CLI Rust y SQLite sintéticos. La revisión cubrió Task sin cuerpo,
formato, `/h1`, duplicación con IDs diferentes, undo/redo, continuación inferior,
reutilización de párrafo vacío, guardado y reapertura, cancelación del borrador,
y continuidad inferior en Note. No hubo solicitudes externas ni errores de página.

El transporte de pruebas sustituye IPC nativo por un puente local a la CLI; no
acredita aceptación de la ventana Tauri ni del cierre nativo del sistema.
