# Proposal

## Why

El menú lateral queda estático en la sesión de desarrollo del usuario. La carga
diferida de AnimateIcons responde 504 Outdated Optimize Dep en el Vite activo;
el catch decorativo lo oculta. Se reproduce sin movimiento reducido y en ambos
motores, por lo que no se atribuye el fallo a WebKit.

## What Changes

- Servir el paquete ESM autocontenido de AnimateIcons fuera del prebundler de
  dependencias de desarrollo, conservando el import diferido y el build actual.
- Conservar un diagnóstico de carga fallida en consola sin bloquear navegación.
- Revisar transformaciones/geometría reales durante el clic en Chromium/WebKit,
  además del indicador de ciclo. Pantalla nativa sólo con autorización específica.
- Sin cambios de iconos, datos, duración, movimiento reducido ni CSP.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

Ninguna. `skip_specs: true`: se corrige la implementación del requisito vigente
«Animación explícita y breve del menú lateral», sin ampliar su contrato.

## Impact

`vite.config.ts`, captura de errores de la acción lateral y documentación de
revisión. Sin nuevas dependencias, operaciones Rust, migraciones o escrituras
sobre el workspace. La corrección de configuración recarga el servidor frontend.
