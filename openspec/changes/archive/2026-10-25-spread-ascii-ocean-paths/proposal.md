# Proposal

## Why

Los cuatro recorridos actuales comparten casi el mismo centro y se acumulan,
como muestra la captura del usuario. Se pide más dispersión y variedad de
posiciones, conservando el aspecto y nado articulado ya aprobados.

## What Changes

- Repartir los centros y alcances de los recorridos por el ancho y distintas
  alturas de la portada, con fases independientes.
- Dar más separación a los tiburones y alternar zonas altas/bajas de los peces
  pequeños, conservando cruces ocasionales y movimiento continuo.
- Conservar mallas, tamaño, paletas, textura, ondulación, pausas y preferencias.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: dispersión espacial sostenida de la portada ASCII.

## Impact

Sólo configuración de recorridos en `shark-scene.ts`, guía y requisito de la
portada. Sin dependencias, operaciones, persistencia ni cambios en otras
pantallas. Verificación geométrica temporal y visual headless en claro/oscuro
y ancho estrecho; no se promete que nunca se crucen las siluetas.
