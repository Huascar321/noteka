# Calendario Due: investigación y morph Liquid Glass

Ajuste posterior del 07/11/2026: a pedido del usuario se reduce la duración de
480/240 a 400/200 ms (apertura/cierre), manteniendo geometría y curvas. El sheen
acompaña el mismo ritmo.

Build y los 56 checks Due por motor (Chromium/WebKit) se repitieron y aprobaron
con este ritmo; se regeneraron los previews de ambos temas. El servidor de
desarrollo abierto sirve las duraciones nuevas, sin reiniciar el workspace.

Fecha: 07/11/2026. Cambio activo: `animate-due-calendar-liquid-morph`.

## Fuentes y conclusión

Apple describe un material y un movimiento diseñados conjuntamente: los menús
surgen de su control y mantienen una relación espacial directa; al crecer,
cambian su profundidad, sombra y comportamiento óptico. No alcanza con añadir
blur y un fade. Fuente primaria: [Meet Liquid Glass, Dynamics, 1:29–6:00](https://developer.apple.com/videos/play/wwdc2025/219/).

En SwiftUI, las presentaciones pueden surgir de su botón y las vistas custom
usan `GlassEffectContainer` e identidades de efecto para combinar y transformar
superficies. Fuentes: [Build a SwiftUI app with the new design](https://developer.apple.com/videos/play/wwdc2025/323/)
y [Applying Liquid Glass to custom views](https://developer.apple.com/documentation/swiftui/applying-liquid-glass-to-custom-views).
Estos mecanismos nativos no son APIs CSS ni especifican aquí las constantes
físicas de la animación del calendario de la referencia.

## Implementación local

- La superficie hermana `.date-picker-material.glass-menu` se mide respecto del
  campo Due y cambia traslación, escala no uniforme y radio. Apertura de 400 ms,
  sobreoscilación de 3,5% horizontal/2,5% vertical y asentamiento amortiguado.
  Los parámetros son de Noteka, ajustados para este panel; no se atribuyen a Apple.
- El contenido tiene una entrada independiente y comienza a aparecer después de
  120 ms, cuando el material ya cubre casi todo el panel. El texto no hereda la
  compresión de la cápsula. El material mantiene el blur y borde compartidos;
  no se animan la intensidad del filtro ni el fondo global.
- Cierre de 200 ms desde el frame visual actual al campo; queda inerte durante
  la salida. Finalización por `Animation.finished`, con fallback de 280 ms.
  Un cierre durante la entrada no salta a la escala final antes de replegarse.
- Foco inicial explícito en Due al abrir con puntero para que Escape funcione
  también en WebKit, donde hacer clic en un botón puede no enfocarlo. La apertura
  de teclado sigue enfocando el día. Se conservan Today/Clear/selección y Save.
- `Reduce Motion` omite entrada y cierre y cancela efectos activos si cambia
  durante la presentación. Contraste alto/transparencia reducida conservan el
  fallback sólido; no se agrega un segundo filtro a la capa de texto.

## Verificación

- `npm run build`: sin errores ni advertencias Svelte. Vite conserva el aviso
  de chunk mayor a 500 kB.
- Recorrido Due: **56 checks aprobados en Chromium y 56 en WebKit**. Selección/guardado con CLI Rust y SQLite
  temporal, teclado, Today/Clear, Escape, clic exterior y segundo clic en Due;
  geometría inicial/final del material, sobreoscilación, cierre temprano sin
  salto, liberación de efectos, 420 px, anclaje arriba y resize abierto.
- Ambos temas y acentos Blue/Violet: el día seleccionado cambia de color y el
  material conserva exactamente su fondo neutro. Contraste aumentado,
  movimiento reducido inicial y cambio de
  preferencia durante una apertura; cero errores de página y cero solicitudes
  externas del frontend.
- [Galería específica](design/due-liquid-morph/README.md): frames reales
  controlados por tiempo, en ambos motores y temas. La primera captura tardía
  podía perder la entrada bajo carga; se corrigió el capturador para congelar
  el primer frame y comprobar que existe el morph antes de exportar.
- El primer recorrido WebKit expuso Escape sin foco al activar con puntero;
  se corrigió en el componente. Los checks geométricos también permitieron
  ajustar el easing por tramo para conservar el tiempo del rebote.

## Límites

No se verificaron IPC/composición nativos WKWebView, pantalla física o 120 Hz,
Windows ni rendimiento GPU. WebKit headless deja ver fondo más nítido que
Chromium; calcular el mismo blur no acredita la misma composición óptica.
Se comprobó la rama sólida mediante contraste aumentado en ambos motores;
no se emuló por separado `prefers-reduced-transparency`. La receta de color
no cambia; las animaciones usan el azul predeterminado y los PNG adicionales
comprueban el segundo acento Violet.

No se ejecutó la suite general: la validación está centrada en el picker y su
persistencia existente. No hay migraciones, dependencias de producto, commit,
push, publicación, sync ni archivo. Los cambios locales anteriores se conservan.
