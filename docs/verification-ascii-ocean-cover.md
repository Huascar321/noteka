# Revisión del océano ASCII

Fecha: 24/10/2026. Cambio `refine-ascii-ocean-cover`.

## Alcance y conservación

Baseline privado de 41 archivos antes de editar, con malla/componente anteriores.
La prueba de búsqueda se copió adicionalmente antes de corregir su fixture.
Se modifican sólo escena/render/reloj, pruebas y documentación. HomePanel,
preferencia ASCII/Foto, estilos globales, WorkspacePane, BlockEditor,
Rust/contratos/locks y funciones anteriores conservan sus bytes.
No se abrió/copió el workspace personal ni se ejecutaron operaciones del núcleo.
Sin dependencias o recursos remotos nuevos.

La escena prepara una malla de 11.024 puntos y proyección por pose sin trigonometría
ni objetos por punto en el bucle de dibujo. Cada tiburón tiene grilla local,
buffers reutilizados y atlas de glifos. La traslación compone su canvas local;
sombreado con histéresis, opacidad cuantizada y limpieza inmediata de vacíos.
Ambiente acotado de36 partículas, doce peces y luz tenue.

## Compilación y checks

`npm run build` aprobado: iconos/contratos y Svelte cero errores/advertencias.
Bundle aceptado `index-CIYZg2kv.js`, CSS `index-BZ84RQPO.css`; Vite conserva el
aviso previo de bundle mayor a 500 kB.

Doce checks aprobados: cinco nuevos de reloj/geometría y siete existentes de
context-search, incluida preferencia/volumen procedural. Se verifican cadencias
30/60/90/120/144/240 con tiempo activo equivalente, máximo120, pausa sin salto,
composición reducida, acotación de una interrupción, proyección finita a varios
tamaños/orientaciones y deformación posterior con morro estable.

La primera ejecución completa canceló cuatro casos porque una fixture anterior
usaba 7 como índice inválido. Con siete rutas, ese índice ya seleccionaba la
primera página y esperaba un deferred sin resolver. Se corrigió sólo el índice
de la prueba para derivarlo de la longitud; no se modificó búsqueda de producto.
La repetición completa terminó 12/12, sin cancelados. Los logs iniciales se
conservan. La primera previsualización tuvo únicamente un 404 de favicon propio;
el harness usa favicon data y la aceptación final no presenta mensajes.

## Revisión visual y rendimiento

Playwright/Chromium revisó el componente real en un harness aislado compilado
con Vite/Svelte de producción, servido por HTTP local. El harness no abre
workspace ni modifica preferencias de la aplicación. Se inspeccionaron capturas
oscura/clara y 420 px; viewport/documento 420/420. Los tres tiburones, partículas,+bancos y luz se distinguen con menor contraste ambiental. No hay ruido de grilla
ni rectángulos detrás de cada carácter.

- Pausa activa, portada fuera de vista y desmontaje/recreación reales aprobados.
- Documento oculto verificado con propiedad/evento simulados; no es aceptación
  de cambio de foco/cierre nativo.
- Movimiento reducido: frame 0 y píxeles exactamente iguales entre capturas
  separadas. Composiciones fijas del renderer también producen píxeles iguales.
- Consola final cero mensajes/errores/advertencias; solicitudes externas cero.
- Bucle real: 210 cuadros en 3,5 s, aproximadamente 60,0 FPS, DPR 1; coste mediano
  1,4 ms y p95 5,3 ms por callback que dibuja. No se midió una pantalla de 120 Hz.

Benchmark aislado con Canvas2D real, doce warmups y 180 dibujos a incrementos de
1/120 s por caso; tiempos de llamada, sin coste final del compositor:

| Tamaño CSS | DPR | Mediana ms | p95 ms | Máximo ms |
| --- | --- | --- | --- | --- |
|1600×288|1|1,2|1,3|1,4|
|1600×288|2|1,2|1,3|1,4|
|420×230|1|0,8|0,9|1,0|
|2400×288|2|1,2|1,3|3,5|

Los tiempos están por debajo de 8,33 ms en este recorrido; no prueban 120 FPS de
presentación en todo equipo. El reloj controlado a 120 Hz tampoco es una medición
física. La referencia inicial en dev queda separada, sin afirmar una mejora
porcentual entre entornos. La captura MP4 dura 8,02 s, 1600×288, muestreo solicitado
hasta 60 y media aproximada 59,5 FPS durante codificación; muestra movimiento,
no una aceptación a 120 Hz.

La primera grabación directa perdió la transparencia del canvas y alteró los
colores al codificar. Se conservó ese intento y se corrigió sólo el harness de
grabación: composición sobre fondo opaco antes de MediaRecorder. Se inspeccionó
un cuadro decodificado del MP4 final; el producto no requiere ese cambio.

## Cierre

El primer comparador de specs usó dot-all también en el título y contó un único
bloque, por lo que falló su aserción. La sincronización ya estaba escrita y el
archivado se ejecutó antes de completar ese contraste. Se corrigió el parser y se
compararon el delta archivado, los 66 bloques restantes y Purpose contra el
baseline exacto, sin resincronizar ni volver a archivar; todo coincide.

Un requisito desktop-shell modificado y contrastado, con sus dos escenarios
anteriores conservados y dos añadidos. Se preservan los otros 66 bloques,
dieciséis specs y ocho artefactos Meeting Notes 0/19. Archivado 4/4 como
`2026-10-24-refine-ascii-ocean-cover`; validación estricta final 18/18.
Capturas, video, logs, baseline y copia aceptada quedan fuera de Git. Se cierran
sólo navegadores/servidor propios del harness; no se reinicia/captura Noteka
nativo ni su Vite, ni se operan Notion, Eco Armario o voicebot.
