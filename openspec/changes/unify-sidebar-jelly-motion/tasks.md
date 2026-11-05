# Tasks

## 1. Motion

- [x] 1.1 Aplicar los keyframes Jelly oficiales de Tasks a las nueve figuras, compartir CSS y actualizar generación/procedencia; verificar que Tasks y catálogo estático se conservan y que la generación es reproducible.
- [x] 1.2 Verificar build, Jelly idéntico por muestras de transformación en los nueve accesos, clic/teclado/reposo/cancelaciones en Chromium/WebKit headless y actualizar guía/plan/evidencia/artefactos.
- [x] 1.3 Ralentizar el ciclo compartido a1000ms con limpieza a1050ms, conservando intensidad/curva/SVG; verificar generación, build y duración/reposo/cancelaciones en Chromium/WebKit, y actualizar documentación.

## Verification

Primera verificación, a700ms: build aprobado, seis checks de generación/preservación y140 headless entre
Chromium/WebKit: idénticas siete muestras de Jelly, origen, duración e iteraciones
en los nueve accesos. Tasks y catálogo estático intactos; clic/teclado, reposo,
carga y cancelaciones aprobados. Ver docs/verification-sidebar-jelly.md. Sin
aceptación nativa nueva ni suites ajenas repetidas; delta pendiente de sync/archivo.

Ajuste posterior a1000ms aprobado: siete checks de generación/preservación,
build y146 headless (73 por motor). CSS difiere sólo en duración; misma curva,
intensidad y figuras. Ciclo activo a800ms y limpieza final verificada. Duración
centralizada en el catálogo; evidencia en la misma guía de verificación.
