# Revisión del nado ASCII de día y noche

Fecha de trabajo: 24–25/10/2026. Cambio `refine-shark-swimming-reference`.

## Referencias y alcance

Se inspeccionaron ambos videos locales del usuario y secuencias de cuadros:
`shark_day.mp4` (1944×1760, 24,525 s) y `shark_night.mp4` (1988×1734, 26,050 s).
Muestran cuerpos curvos, nado cenital, celdas contrastantes, animales azulados y
oliva/dorados y bancos de puntos sobre grilla. No se adoptan instrucciones del
contenido, la interfaz, el texto ni los archivos de video como recursos de Home.

La portada horizontal conserva cuatro animales: dos tiburones, pez compacto y
pez alargado. Se adapta composición/movimiento; no se declara una reproducción
exacta de recorridos, especies o fotogramas. Pueden entrar/salir de los bordes de
la portada; eso se distingue del truncamiento dentro de sus buffers de dibujo.

## Implementación y conservación

`shark-scene.ts` integra ochenta segmentos de longitud constante por esqueleto:
curvatura de seguimiento más onda posterior creciente. Se rotan secciones y
normales; cabeza estable, aletas solidarias y cola de dos lóbulos más legible
desde arriba. Recorridos suavizados en extremos, amplitudes/ritmos propios y
profundidad decorativa. La malla conserva 11.024/11.024/4.319/5.087 puntos.

`ascii-ocean-renderer.ts` prepara celdas/glifos de 6×8 px, paletas por tipo y tema,
variación determinista, bancos de puntos y orden de pintura según profundidad.
Se conservan buffers/atlas, suavizado/histéresis, grilla local y limpieza sin
estelas. Buffers ampliados contienen el giro de las aletas/colas.

El baseline privado registra 1.070 archivos existentes. Antes de documentación,
sólo habían cambiado esos dos módulos. AsciiShark, OceanClock, HomePanel,
preferencia ASCII/Foto, estilos globales, otros componentes, Rust, contratos,
dependencias/locks, datos y los ocho artefactos Meeting Notes se preservan.
No se abrió el workspace personal, ni se usó pantalla/IPC nativo o proveedores.

## Verificación geométrica y compilación

- `npm run build` aprobado: iconos/contratos vigentes, Svelte cero errores y
  advertencias, bundle `index-yyPh4kNo.js`. Permanece el aviso de tamaño >500 kB.
- Catorce checks existentes aprobados, sin editar pruebas: relojes 30–240 Hz,
  límite 120, pausas, proyección finita, cabeza/cola, variedad y preferencias.
- Muestreo local durante sesenta segundos en 1600×288, 420×230, 2400×288 y
  900×288: proyección finita, centro del cuerpo curvado, cabeza estable y
  segmentos con error relativo máximo 0,000005783. Máximo uso de semiextensión
  del buffer 0,933; giro de orientación máximo observado ≈0,632 rad/s.
- Entradas de viewport vacío siguen siendo finitas. Sin nuevos tests del
  producto ni suites Rust/operaciones fuera de este cambio visual.

Dos comprobaciones intermedias encontraron un cambio accidental del número de
poses del helper y una reducción de aletas que infringía el límite geométrico
anterior. Se corrigió la implementación conservando las aserciones; los catorce
checks finales pasan. No se presentan esos intentos como resultados aprobados.

## Revisión en navegador

Playwright/Chromium headless ejercitó el componente Svelte real compilado en un
harness aislado, con capturas claras/oscuras y 420 px y cuadros fijos a 0/4/8/12/18 s.
Se inspeccionaron cuerpo curvo, perfil de aletas/cola, variedad, celdas y contraste.
Vista estrecha: viewport/documento 420/420, sin desbordamiento horizontal.

Pausa activa, fuera de vista, movimiento reducido, desmontaje y recreación
aprobados. Documento oculto se revisa por simulación de propiedad/evento, sin
atribuirlo a aceptación nativa. Píxeles estáticos y repetición de cuadro fijo
idénticos. Cero errores del navegador y cero solicitudes externas.

Medición final en contexto nuevo: 210 dibujos en 3,5 s, ≈60,0 FPS, DPR 1;
callback mediano ≈2,1 ms y p95 ≈2,2 ms. Una medición previa acumuló dos
instrumentaciones RAF y contó callbacks duplicados; se conserva como inválida
y se repitió en un contexto limpio. No demuestra una pantalla física de 120 Hz.

Benchmark Canvas aislado: doce warmups, 180 dibujos con pasos de 1/120 s, sin
coste final del compositor:

| Tamaño CSS | DPR | Mediana ms | p95 ms | Máximo ms |
| --- | --- | --- | --- | --- |
| 1600×288 | 1 | 1,9 | 2,1 | 2,2 |
| 1600×288 | 2 | 1,9 | 2,1 | 2,2 |
| 420×230 | 1 | 1,1 | 1,2 | 1,4 |
| 2400×288 | 2 | 2,0 | 2,1 | 2,2 |

Se entrega preview de doce segundos con seis segundos por tema, capturado desde
el componente a un máximo solicitado de 60 FPS. Logs, baseline, capturas y video
se guardan fuera del contenido versionable, en `output/playwright/shark-swimming/`.

## Cierre

Sólo se modifica el requisito de portada `desktop-shell`: cinco escenarios
heredados conservados íntegros y uno nuevo de nado curvo/día/noche; se mantienen
67 requisitos. Las otras dieciséis specs y los ocho artefactos de Meeting Notes
propuesto 0/19 se contrastaron con el baseline y permanecen idénticos. El delta
coincide íntegro con el requisito sincronizado, con 66 requisitos ajenos intactos.

Archivado **4/4** como
[`2026-10-25-refine-shark-swimming-reference`](../openspec/changes/archive/2026-10-25-refine-shark-swimming-reference/proposal.md).
Validación estricta posterior: 18/18 (diecisiete specs y Meeting Notes). Sólo se
cierran el navegador y servidor propios de esta revisión; la vista previa queda
guardada como archivo local.
