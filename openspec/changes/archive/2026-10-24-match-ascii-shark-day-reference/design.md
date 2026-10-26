# Design

## Context

El video local dura24,525 s y se inspeccionó en siete momentos. La referencia usa vista cenital, celdas grises/oliva con glifos claros/oscuros, pocos acentos pastel, grilla tenue y cuerpos que se curvan/entran/salen del encuadre. El usuario precisó que hay otros peces. La portada horizontal requiere adaptar esa composición sin copiar controles/texto del video.

## Goals / Non-Goals

Acercar apariencia y movimiento; incluir variedad real de mallas. Preservar120 FPS máximos, pausas, preferencia/controles, datos y funciones. No reproducir fotogramas, integrar el video como fondo ni añadir controles o proveedores.

## Decisions

1. Cámara más cenital y posterior más largo/flexible para tiburones. Mantener API geométrica actual y preparar dos mallas adicionales: pez compacto con cuerpo ancho/aletas cortas y pez alargado con cuerpo estrecho/cola bifurcada. El elenco contiene dos tiburones y uno de cada pez; cada tipo tiene amplitud/frecuencia de nado y recorrido propios. Las etiquetas son categorías visuales, no una identificación biológica de especies del video.
2. Atlas de celdas completo por tema: fondo sombreado, letra contrastante, gris/oliva y pequeños acentos pastel deterministas ligados a cada grilla local. Histéresis y opacidad cuantizada siguen evitando ruido temporal; bordes menos opacos. Reutilizar buffers y proyección preparada, ampliando límites locales para no truncar la cola al girar.
3. Cambiar rayos por grilla fina preparada una vez y partículas/bancos lejanos como pequeños trazos. Mantener el ambiente subordinado a los animales y al contenido de Home. Sin nuevos fetch, imágenes, datos o preferencias.
4. Conservar AsciiShark/OceanClock/HomePanel/preferencia y sus condiciones. Revisar el componente compilado en harness aislado con Playwright, métricas Canvas separadas del FPS físico y video sobre fondo opaco. No abrir el workspace ni la ventana nativa.

## Risks / Trade-offs

- Celdas rellenas y sujetos más grandes aumentan el coste del dibujo; medir con DPR1/2 y420 px antes de cerrar.
- Una portada baja recorta algunas entradas/salidas de animales; mantener buffers locales suficientemente grandes y distinguir recorte del encuadre de truncado de geometría.
- La referencia no determina una especie exacta para cada forma; implementar variedad visual sin atribuciones taxonómicas.

## Migration Plan

Sin migración, contratos, Rust, dependencias ni datos modificados. Baseline privado de47 archivos y evidencia del video fuera de Git. Mantener el reloj, Home, demás detalles y Meeting Notes0/19 intactos. Sólo se sincroniza comportamiento verificado de la portada.
