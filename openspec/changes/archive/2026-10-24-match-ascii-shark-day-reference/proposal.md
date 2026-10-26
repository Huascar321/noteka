# Proposal

## Why

El usuario aprobó la mejora anterior y pidió acercar los tiburones a su video local de referencia. Se inspeccionaron cuadros a0/2/5/8/12/16/20 s: vista cenital, cuerpos largos que se curvan, volumen por celdas rellenas, letras contrastantes y pocos acentos de color sobre grilla tenue. El resultado anterior usa glifos sueltos y rayos que no corresponden a esa referencia.

## What Changes

- Cámara más cenital, posterior alargado y curvatura creciente; sujetos principales más presentes en el encuadre de la portada.
- Celdas sombreadas con glifos de contraste, variación determinista y acentos oliva/azul/verde discretos en ambos temas.
- Combinar dos tiburones con peces de cuerpo compacto/alargado, usando mallas y nado distintos; no sólo escalas del mismo animal.
- Fondo con grilla fina y partículas/bancos lejanos de trazos pequeños; retirar los rayos triangulares.
- Conservar atlas/buffers, estabilidad temporal, reloj120, pausas, movimiento reducido y controles ASCII/Foto.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: apariencia y movimiento de la portada ASCII guiados por el video del usuario.

## Impact

Sólo malla/poses, renderer Canvas2D y documentación. Sin cambiar HomePanel, reloj, contratos, datos, dependencias o controles. Referencia y capturas fuera de Git; el video no se incorpora como recurso del producto. Compilación, checks existentes y nuevos de diversidad geométrica y revisión/rendimiento del componente compilado con Playwright.
