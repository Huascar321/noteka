# Design

## Context

Wry en macOS entrega el drop al WebView sólo si el manejador nativo lo rechaza. Tauri instala por defecto uno que acepta los eventos, incluso para arrastres internos; Noteka no consume esos eventos de archivos. El navegador aislado sí ejecuta el movimiento existente.

## Goals / Non-Goals

Restaurar el requisito vigente de arrastre HTML y conservar la protección frente a arrastres externos. No cambiar reglas del tablero, contratos, persistencia ni soporte de importación de archivos.

## Decisions

Desactivar `dragDropEnabled` en la ventana y prevenir por defecto el drop de archivos o URLs externos en la interfaz. Los arrastres internos de proyectos y bloques conservan su propio manejo. Se reutilizan las operaciones de lote y CAS de Rust/SQLite; no hay migración, ni proveedor remoto.

## Risks / Trade-offs

La configuración requiere reiniciar la ventana nativa; la prueba WebKit aislada no acredita por sí sola la integración Tauri. Probar los gestos con datos sintéticos y registrar ese límite.

## Seguimiento 29/10/2026

El usuario informa que el gesto sigue sin funcionar en la ventana nativa. El tablero usa eventos de puntero para las tarjetas, con umbral de movimiento, destino visible, cancelación y preservación de clics y del diálogo de teclado. La posición sigue pasando por el mismo lote con CAS. El drag HTML y la protección de drops externos permanecen para el editor y los archivos.

La tarjeta completa inicia el gesto, incluidas sus relaciones y el botón Move;
un clic sin superar el umbral conserva la acción del elemento. El hover de los
botones internos no pinta un rectángulo parcial sobre la tarjeta.

El contenedor desplazable reserva espacio alrededor de las columnas para que el
contorno exterior de destino no quede recortado en los extremos del tablero.
