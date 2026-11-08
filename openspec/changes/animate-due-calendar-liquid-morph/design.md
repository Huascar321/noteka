# Design

## Context

`DueDatePicker.svelte` ya posiciona el panel y coordina cierre/foco. La versión previa aplica escala .58 uniforme desde una esquina durante 420 ms. La interfaz es Svelte/WKWebView, no SwiftUI; SQLite y Rust conservan autoridad sobre las operaciones existentes. Ver `proposal.md` para la motivación.

## Goals / Non-Goals

**Goals:** continuidad geométrica campo-panel, deformación acotada del material y texto sin compresión.

**Non-Goals:** equivalencia óptica nativa, nuevo renderer, dependencia, reloj permanente o modificación de otros pickers.

## Decisions

- Medir el campo y el panel al posicionar. Una superficie absoluta `.date-picker-material.glass-menu` se transforma desde el rectángulo del campo al del panel. El contenedor semántico permanece fijo y el contenido va en una capa hermana: evita comprimir números o alterar layout por frame.
- Usar Web Animations API con keyframes de expansión y amortiguación (400 ms) y repliegue (200 ms). Elegimos keyframes finitos frente a simulación continua o una biblioteca de resortes; la curva es una elección de Noteka, no una constante publicada de Apple.
- Capturar el estado visual actual antes de cancelar una entrada para cerrar sin salto. Al iniciar cierre se vuelve a medir el destino actual; las acciones quedan inertes y no se repiten selecciones.
- Mantener una única superficie filtrada local y el borde compartido. El contenido usa fundido y traslación pequeña; el sheen es breve y no se repite en reposo. Las preferencias se respetan al abrir y al cambiar mientras está montado.
- Fuente primaria: [Meet Liquid Glass, Dynamics](https://developer.apple.com/videos/play/wwdc2025/219/), explicación del menú que surge de su botón; [Build a SwiftUI app with the new design](https://developer.apple.com/videos/play/wwdc2025/323/), presentaciones desde controles y contenedores de vidrio; [Applying Liquid Glass to custom views](https://developer.apple.com/documentation/swiftui/applying-liquid-glass-to-custom-views), `GlassEffectContainer`/identidad para morph nativo. Apple compone luz, lente y material; el ensayo aproxima geometría y movimiento, sin afirmar refracción.

## Risks / Trade-offs

- Transformar el material también transforma su blur y borde → comprobar Chromium/WebKit y registrar límite de composición WKWebView.
- Cierre temprano o desmontaje → capturar antes de cancelar, fallback temporal y limpieza de animaciones.
- El botón sigue visible bajo el origen durante la expansión → se conserva su semántica/foco; no se duplica texto en el material.
- Animar radio produce trabajo de pintura local → duración y superficie pequeñas, sin animar blur ni crear RAF permanente; no se promete tasa física de frames.

## Migration Plan

Sin migración ni datos nuevos. Restaurar sólo los hunks de este cambio revierte la presentación; no revertir cambios locales anteriores. Se verifica localmente y no se publica ni archiva en esta tarea.
