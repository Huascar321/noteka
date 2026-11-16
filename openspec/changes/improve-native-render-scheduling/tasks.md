# Tasks

## 1. Reproducción de overlays

- [x] 1.1 Preparar recortes fuera del tick y reproducir WAAPI sin seeks por frame; documentar y verificar geometría, tiempos 400/200, interrupción, buffers y preferencias con pruebas focales y capturas.

## 2. Scroll y canvas

- [x] 2.1 Clasificar wheel pasivamente y limitar cancelación al scroller discreto; verificar suite existente de rueda y gestos/bordes/limpieza sin listener global bloqueante.
- [x] 2.2 Acotar diagnóstico e invalidación del canvas sin bajar velocidad ni pausar tras popups; verificar escenas/pausas/resize y medición antes/después.
- [x] 2.3 Verificar y optimizar apertura de Side peek: transform 260 ms, buffers coalescidos, una lectura fresca por fila y metadatos no bloqueantes con pruebas de borradores/respuestas tardías.

## 3. Integración y evidencia

- [x] 3.1 Ejecutar build, mediciones comparables Chromium/WebKit y verificación nativa; conservar trazas/capturas, actualizar guía/plan/workflow, registrar límites y validar OpenSpec/diff.

## Workflow follow-up

- Sin commit de rendimiento, push ni archivo automático solicitados; el commit de vidrio ya está hecho.
