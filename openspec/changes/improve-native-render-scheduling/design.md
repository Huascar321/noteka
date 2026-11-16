# Design

## Context

El perfil nativo previo sobre Home contiene Canvas2D/getImageData, resolución de estilo y recomposición. La portada ya tiene ajustes locales no confirmados de cadencia y SVG; se conservan. `liquid-overlay` fuerza currentTime y lee rectángulos/estilo en cada tick. `smooth-wheel` registra wheel no pasivo en document incluso para trackpad.

## Goals / Non-Goals

**Goals:** reducir trabajo repetido en el hilo de UI y liberar scrolling asíncrono; comparar evidencia antes/después.

**Non-Goals:** acelerar duraciones, retirar vidrio o portada, cambiar escenas/datos/operaciones, incorporar workers o cambiar CSP en este recorte.

## Decisions

- Calcular recortes desde los keyframes de material y geometría estable al comenzar; reproducir material, opacidad y recorte con Web Animations. Leer el estado actual sólo para interrupción o reposicionamiento real. La salida sigue capturando el frame actual. Se conserva la restauración de recortes originales y limpieza por preferencias.
- La clasificación global de wheel es pasiva y en captura; sólo una rueda discreta arma un listener cancelable en el scroller antes de su fase de burbujeo. Un evento nativo no cancelable se deja pasar. Gestos finos, límites, interrupción e inactividad liberan el enlace local.
- Limitar escrituras de diagnóstico del canvas y aislar su superficie de pintura/composición, conservando RAF/tiempo activo y cadencia existentes. No se atribuye a este cambio el trabajo previo de weather/cadencia.
- El perfil nativo posterior sigue mostrando esperas de IPC en `getImageData`. Se ensayó agrupar las máscaras para una lectura y usar OffscreenCanvas en el mismo hilo: ambas rutas conservaron los píxeles en 48 casos por motor ensayado, pero no mejoraron el tiempo de dibujo. Se descartaron; no se publica una reducción de llamadas como mejora de rendimiento cuando el costo medido aumenta. El renderer conserva su arquitectura y ese costo conocido.
- Comparar mediciones de APIs, intervalos y muestras nativas; contadores headless no equivalen a una tasa física de pantalla. Mantener velocidad/geometría y casos de foco/borrador en las suites existentes.
- Datos y reglas siguen en Rust/SQLite; no hay migración ni dependencia nueva.
- Side peek conserva 260 ms y el ancho final del dueño, pero su deslizamiento usa transform en vez de right. El canvas conserva su buffer durante el cambio de dimensiones y lo reconstruye una vez tras 90 ms sin resize; sigue dibujando durante el movimiento. La fila entrega sólo kind/title como pista de navegación: el panel siempre obtiene el original fresco. Relaciones/dashboard/recursos se leen en paralelo sin bloquear el editor ya cargado; generaciones, abort y selección vigente impiden aplicar resultados de otra apertura.

## Risks / Trade-offs

- [Recorte difiere del vidrio] → keyframes con el mismo reloj, muestras geométricas densas y revisión de frames en ambos motores.
- [Primer evento nativo no cancelable] → conservar su scroll nativo y armar sólo eventos cancelables posteriores; verificar dispositivo simulado y rueda real del browser.
- [Aislamiento cambia recorte o tamaño] → limitarlo al canvas de dimensiones explícitas; capturas/resize y portada detrás de overlays.
- [Mediciones varían por carga/escena] → declarar contexto, conservar resultados y no prometer 120 Hz físicos ni atribución única del problema.
