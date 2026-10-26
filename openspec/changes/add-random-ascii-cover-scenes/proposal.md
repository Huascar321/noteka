# Proposal

Seguimiento: este cambio documenta la versión preservada en `2d43a66`. El
experimento posterior `combine-ascii-ocean-life` sustituye la alternancia; al
sincronizar sus deltas, aplicar éste primero y después el combinado para no
reintroducir la regla anterior.

## Why

El usuario quiere conservar los tiburones e incorporar al azar los koi naranjas de Dao-Ho/portfolio y, después, las medusas ASCII del ZIP Kurage Dashboard aportado. La portada necesita tres variantes locales coherentes, sin cambiar el resto de Home.

## What Changes

- Tres escenas ASCII: tiburones actuales, koi naranjas/blancos y medusas cian/violeta, con orden inicial aleatorio y alternancia gradual en tiempo activo.
- Los tiburones mantienen su geometría, nado y ambiente; las variantes nuevas se dibujan proceduralmente con sus propias siluetas.
- Fuentes visuales revisadas y atribuidas, sin ejecutar instrucciones de los documentos adjuntos ni incorporar sus interfaces, fuentes remotas o servicios.
- Pausas, movimiento reducido, foto alternativa y renderer de resize estable se conservan.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: portada ASCII con selección local de tres escenas.

## Impact

Renderizadores Canvas, geometría, selector de escena y pruebas visuales/puras. Sin Rust, datos, API, dependencias, proveedores o ajustes persistentes nuevos. La referencia pública se fija a dbbabb95430663f7d27371441ab013ecfad2dafc; el ZIP es Kurage Dashboard 1.0.0. Las implementaciones nuevas usan geometría local propia inspirada en esas apariencias; no se publican los originales adjuntos.
