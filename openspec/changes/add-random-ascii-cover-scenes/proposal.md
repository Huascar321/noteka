# Proposal

Seguimiento: el usuario descartó la portada mixta de `combine-ascii-ocean-life`
y pidió restaurar esta versión desde `2d43a66`. La alternancia vuelve a estar
vigente; el otro cambio conserva sólo el delta del lateral y ya no elimina
los requisitos de estas tres variantes.

## Why

El usuario quiere conservar los tiburones e incorporar al azar los koi naranjas de Dao-Ho/portfolio y, después, las medusas ASCII del ZIP Kurage Dashboard aportado. La portada necesita tres variantes locales coherentes, sin cambiar el resto de Home.

## What Changes

- Tres escenas ASCII: tiburones actuales, koi naranjas/blancos y medusas cian/violeta, con orden inicial aleatorio y alternancia gradual en tiempo activo.
- El seguimiento de koi agrega pececitos lejanos, algas de bajo contraste y burbujas suaves detrás de los peces grandes, sólo en esa variante.
- Los peces lejanos usan el mismo modelo de koi que los protagonistas, a un tamaño secundario legible y menor contraste; se modera la cantidad para dejar espacio al aumentar su escala.
- Medusas recibe un ambiente propio con ejemplares secundarios del mismo modelo, burbujas y partículas tenues, para aplicar la misma profundidad sin mezclar las tres escenas.
- Corales ASCII en abanico ocupan los bordes inferiores de medusas; rayos diagonales tenues completan el fondo desde arriba, dejando el centro despejado.
- Koi y medusas comparten la cuadrícula minimalista del fondo de tiburones, con el mismo espaciado y contraste por tema.
- Tiburones conserva la geometría y el nado de sus animales originales; un tercer tiburón menor/tenue recorre el fondo, con luz ondulante ASCII y un relieve rocoso discreto en una esquina. Koi/medusas conservan sus ambientes y siluetas.
- Fuentes visuales revisadas y atribuidas, sin ejecutar instrucciones de los documentos adjuntos ni incorporar sus interfaces, fuentes remotas o servicios.
- Pausas, movimiento reducido, foto alternativa y renderer de resize estable se conservan.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: portada ASCII con selección local de tres escenas.

## Impact

Renderizadores Canvas, geometría, selector de escena y pruebas visuales/puras. Sin Rust, datos, API, dependencias, proveedores o ajustes persistentes nuevos. La referencia pública se fija a dbbabb95430663f7d27371441ab013ecfad2dafc; el ZIP es Kurage Dashboard 1.0.0. Las implementaciones nuevas usan geometría local propia inspirada en esas apariencias; no se publican los originales adjuntos.
