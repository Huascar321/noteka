# Proposal

## Why

La serie Liquid Glass aceptada ya cubre navegación, herramientas, People, notas y popups, pero la guía no tiene capturas versionadas ni un procedimiento autosuficiente para reproducirla. El usuario pide consolidar todas las pautas y guardar la serie completa en un commit local.

## What Changes

- Ampliar `docs/design-system.md` con recetas, tokens, medidas, estados, accesibilidad, restricciones y errores conocidos de la implementación vigente.
- Agregar una galería de capturas reales del frontend con datos sintéticos y un generador reproducible con Playwright y el puente local existente.
- Registrar procedencia, motores, dimensiones y límites de las capturas; enlazar la evidencia funcional previa sin presentarla como nueva aceptación nativa.
- Actualizar el seguimiento de los siete cambios visuales para reflejar la autorización de commit local y consolidar código, documentación y evidencia en ese commit.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

Ninguna. `skip_specs: true`: documentación y herramientas de captura; no cambia comportamiento del producto.

## Impact

Guía, assets PNG/manifest, scripts de captura, package.json y seguimiento OpenSpec. Se incluyen en el commit los cambios Liquid Glass ya implementados. Sin dependencias nuevas, datos personales, publicación, sync ni archivo automático.
