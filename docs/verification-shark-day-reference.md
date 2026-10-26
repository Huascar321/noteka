# Revisión de la referencia de peces ASCII

Fecha: 24/10/2026. Cambio `match-ascii-shark-day-reference`.

## Referencia y alcance

Se inspeccionó el video local aportado por el usuario: 24,525 s, 1944×1760 y
aproximadamente 57,5 FPS. Se extrajeron y observaron cuadros a 0/2/5/8/12/16/20 s.
La referencia muestra vista cenital, cuerpos curvos, celdas rellenas con letras
contrastantes, acentos discretos y grilla fina. El usuario añadió explícitamente
la variedad de peces. La interfaz y el texto del video no se copiaron.

La portada horizontal adapta esa composición; no busca reproducir exactamente
sus recorridos o cada especie. Se incluyen dos tiburones y peces de cuerpo
compacto/alargado con mallas propias, no copias escaladas. Los nombres describen
perfiles visuales. El video, su SHA-256 y capturas quedan fuera de Git.

## Implementación y conservación

- Tiburón: 11.024 puntos, posterior más largo/flexible y cámara más cenital.
  Pez compacto: 4.319 puntos; pez alargado: 5.087. El elenco proyecta 31.454 puntos
  por cuadro con amplitudes/frecuencias y trayectorias propias.
- Atlas de celdas con fondo, letra contrastante, sombreado y acentos deterministas;
  tonos por tipo y tema. Grilla de fondo preparada y pequeños trazos lejanos;
  se retiran los rayos triangulares. Buffers locales más altos evitan truncar
  colas por el giro dentro del canvas del animal.
- Se conservan exactamente AsciiShark, OceanClock, HomePanel, preferencia,
  WorkspacePane, BlockEditor, CSS global, contratos, Rust, locks y dependencias.
  Baseline privado de 47 archivos; 44 seguían idénticos antes de documentación.
- No se abrió/copió el workspace personal ni se ejecutaron operaciones del núcleo.
  No hay recursos remotos ni cambios de datos o preferencias.

## Compilación y checks

`npm run build` aprobado, con iconos/contratos y Svelte cero errores/advertencias.
Bundle aceptado `index-wvcIut6r.js`, CSS `index-BZ84RQPO.css`; Vite conserva el
aviso anterior de bundle mayor a 500 kB.

Catorce checks aprobados: doce anteriores y dos de variedad geométrica/nado.
Se verifican proporciones distintas, proyección finita, ritmos independientes,
cola deformable con morro estable, reloj a 30/60/90/120/144/240, pausas y
preferencia de portada. El primer recorrido encontró que el check anterior
usaba x=-1,7 como cola; después de alargar el posterior ese punto ya no era la
cola. Se cambió la muestra al punto caudal real, conservando el umbral anterior.
Los logs iniciales y finales permanecen en la evidencia privada.

## Revisión visual y rendimiento

Playwright/Chromium revisó el componente real compilado en harness aislado.
Se inspeccionaron capturas claras/oscuras y de 420 px, comparándolas con la
referencia. Se distinguen los cuatro animales, los perfiles más compactos y
alargados, celdas/letras y fondo discreto. Viewport/documento: 420/420.

Pausa activa, fuera de vista, desmontaje/recreación y movimiento reducido
aprobados. Documento oculto se simula mediante propiedad/evento, sin atribuirlo
a aceptación nativa. La composición estática y los cuadros fijos del renderer
producen píxeles exactamente iguales. Consola: cero mensajes/errores/advertencias;
solicitudes externas: cero.

Bucle real: 210 cuadros en 3,5 s, unos 60,0 FPS, DPR 1; coste mediano 1,6 ms y
p95 1,8 ms por callback que dibuja. El límite de 120 se conserva; no se midió
una pantalla física de 120 Hz.

Benchmark Canvas aislado, doce warmups y 180 dibujos por caso a incrementos de
1/120 s, sin coste final del compositor:

| Tamaño CSS | DPR | Mediana ms | p95 ms | Máximo ms |
| --- | --- | --- | --- | --- |
| 1600×288 | 1 | 1,5 | 1,6 | 2,0 |
| 1600×288 | 2 | 1,5 | 1,6 | 4,3 |
| 420×230 | 1 | 0,8 | 0,9 | 1,0 |
| 2400×288 | 2 | 1,5 | 1,6 | 1,8 |

Estos tiempos no prueban 120 FPS físicos en todo equipo. La vista previa se
capturó en modo claro sobre fondo opaco, con muestreo solicitado hasta 60 FPS;
es una demostración de la animación, no una aceptación de pantalla de 120 Hz.
No se reinició/capturó Noteka nativo, ni se operaron Notion u otros proyectos.

## Cierre

Un requisito desktop-shell modificado y contrastado: 67 requisitos totales,
66 bloques no afectados y cuatro escenarios heredados conservados, más uno
nuevo. Las otras dieciséis specs y ocho artefactos Meeting Notes permanecen
idénticos; Meeting Notes sigue propuesto 0/19. Archivado 4/4 como
`2026-10-24-match-ascii-shark-day-reference`; validación estricta final 18/18.
Evidencia, video y copia aceptada fuera de Git; sólo previews/servidor propios
se cierran al terminar.
