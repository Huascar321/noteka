# Tasks

## 1. Presentation

- [x] 1.1 Fijar texto/atajos blancos en botones primarios compartidos, conservar fondos/estados y verificar build y CSS de producción en claro/oscuro, registrando evidencia y alcance.

## Verification

26/10/2026: build aprobado, iconos/contratos vigentes y Svelte con cero errores/
advertencias; permanece el aviso previo de tamaño del bundle. Revisión aislada
headless con CSS de producción y paletas reales:30 checks aprobados en ambos
temas para New, Save y acción primaria genérica. Texto y atajo rgb(255,255,255),
atajo opacity1, fondo normal/hover conservado (#3183a6 con la selección inicial),
disabled conserva opacity0.5/bloqueo y el botón secundario conserva su tinta.
Capturas, valores y build en output/playwright/button-text-20261026 (ignorado).
El primer intento leyó una transición de color antes de asentarse; el driver
espera el estado estable y compara hover con la paleta, sin cambiar producto.
Sin tests permanentes, suites funcionales repetidas ni aceptación nativa nueva.
