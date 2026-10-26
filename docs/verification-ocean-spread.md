# Revisión de dispersión del océano ASCII

Fecha: 25/10/2026. Cambio `spread-ascii-ocean-paths`.

La captura del usuario muestra varios animales acumulados a la derecha. Sus
centros de recorrido anteriores estaban entre el 40 % y el 54 % del ancho,
con radios superpuestos. Se modifican únicamente centros, radios y una fase en
`shark-scene.ts`: tiburones repartidos izquierda/derecha y peces pequeños en
zonas altas/bajas. Continúan explorando 32 %/34 % del ancho y 42 %/26 % de la
altura respectivamente, con su mismo ritmo y nado articulado.

No cambian mallas, tamaño, renderer, materiales, paletas, reloj, componente,
Home, preferencias, Rust, contratos, dependencias ni datos. No se usa el
workspace personal ni pantalla nativa. El baseline permite contrastar todos
los archivos ajenos, incluidas las otras dieciséis specs y Meeting Notes 0/19.

## Verificación

- `npm run build` aprobado, Svelte cero errores/advertencias; bundle
  `index-CASmF35l.js`, con el aviso anterior de tamaño >500 kB.
- Catorce checks existentes aprobados, sin editar ni añadir tests del producto.
- Treinta minutos simulados por tamaño, 7.201 muestras cada 0,25 s: cercanía de
  tres centros en una región de <22 % del ancho y <50 % de la altura pasa de
  47,16 % a 0,18 % en 1600×288; de 47,90 % a 0,014 % en 420×230; de 47,22 % a
  0,11 % en 2292×700. Es un indicador comparativo de agrupación de centros,
  no una medición de colisión de siluetas ni una promesa de cero cruces.
- Separación horizontal de centros de tiburones >=22 % del ancho por sus
  límites de recorrido; muestreo adicional a 120 Hz durante dos minutos por
  tamaño conserva posiciones finitas, amplitudes y orientación continua.
  Giro máximo observado ≈0,624 rad/s; sin saltos o estado de simulación nuevo.
- Playwright/Chromium headless con componente Svelte real compilado: temas,
  420 px, capturas a 0/15/30/60/120 s y preview animado. Se inspecciona reparto
  y variación de posiciones. Viewport/documento 420/420.
- Pausa activa, fuera de vista, movimiento reducido, desmontaje y recreación
  aprobados; documento oculto simulado por propiedad/evento. Píxeles estáticos
  idénticos, cero errores y cero solicitudes externas.
- 210 cuadros en 3,5 s, ≈60 FPS observados; callback p95 ≈2,4 ms. Canvas aislado
  p95 <=2,3 ms en los tamaños revisados a DPR 1/2. Sin aceptación física 120 Hz.

Se conserva evidencia privada/ignorada en `output/playwright/ocean-spread/`,
incluidos baseline, indicadores, capturas y preview. El cambio sincroniza sólo
el requisito de portada: seis escenarios heredados íntegros y uno añadido,
con los otros 66 requisitos desktop-shell conservados. Archivado **3/3** como
[`2026-10-25-spread-ascii-ocean-paths`](../openspec/changes/archive/2026-10-25-spread-ascii-ocean-paths/proposal.md),
con validación estricta final 18/18. Se cierran sólo navegador/servidor propios;
la vista previa permanece disponible como archivo local.
