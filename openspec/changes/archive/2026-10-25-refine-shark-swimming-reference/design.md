# Design

## Context

Ver motivación y alcance en `proposal.md`. Las mallas locales actuales usan un
desplazamiento lateral por vértice, sin rotar secciones ni sus normales; por eso
el giro conserva un tronco rígido. El atlas actual es predominantemente oliva.
`AsciiShark` y `OceanClock` ya resuelven cadencia, visibilidad, movimiento reducido
y desmontaje. Home es una portada horizontal, distinta de los videos casi cuadrados.

## Goals / Non-Goals

**Goals:** articulación continua sin alargar el esqueleto; reutilizar geometría,
buffers y atlas; comparar ambos videos y la composición horizontal/estrecha.

**Non-Goals:** reproducción exacta de fotogramas, especies biológicas, interfaz
del video, nueva preferencia, GPU/3D externo, datos de usuario o aceptación nativa.

## Decisions

- Esqueleto longitudinal muestreado con longitud de segmento constante. Cada
  sección gira sobre una línea central que combina curva de seguimiento del giro
  y onda posterior. Rotar puntos y normales juntos evita estirar la cola y deja
  que aletas/luz acompañen el cuerpo. Frente estable, amplitud creciente atrás.
  Se descarta sólo aumentar la traslación lateral porque deforma longitudes.
- Recorridos elípticos suavizados con menos velocidad angular en extremos y
  curva proporcional a la trayectoria, acotada para evitar pliegues. Tipos con
  ritmo, profundidad y escala propios; pintar de lejos a cerca al cruzarse.
- Atlas fino de glifos/celdas por tipo y tema: gris azulado, oliva y dorado.
  Grano/acento determinista y sombreado continuo con histéresis existente;
  bancos de puntos discretos reutilizando descriptores preparados.
- Mantener dueño de datos, núcleo y operaciones exactamente iguales: esto es
  presentación sin persistencia. No hay proveedor, acceso remoto ni migración.
- Revisar componente Svelte compilado en harness headless aislado y medir dibujo
  separado de FPS físicos. Registrar capturas/preview en directorio ignorado.

## Risks / Trade-offs

- Curvas muy cerradas o canvas horizontal → limitar articulación, inspeccionar
  fin/cola en distintos ángulos y mantener margen en buffers locales.
- Más celdas y articulación → trigonometría por sección, buffers reutilizados y
  medición a DPR 1/2, ancho normal/estrecho; no prometer 120 FPS físicos.
- Silueta ASCII discreta → inspección visual en tiempos fijos y movimiento;
  contraste por tema e histéresis, sin ruido nuevo por cuadro.

## Migration Plan

No cambia ningún formato. Aplicar sólo módulos de presentación; revertirlos con
el baseline privado restaura la portada anterior sin tocar datos/preferencias.
Sincronizar únicamente el requisito verificado de portada antes de archivar.
