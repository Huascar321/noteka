# Proposal

## Why

Cada arranque de la aplicación muestra la pantalla de bienvenida porque el
workspace abierto vive sólo en memoria: `App.svelte` inicializa `workspace` en
`null` y el usuario debe crear o abrir uno cada vez. Un usuario con un workspace
existente en el directorio de datos vuelve a pasar por la misma pantalla sin
elegir nada distinto.

## What Changes

- Al iniciar, la aplicación abre automáticamente el último workspace usado y,
  si no hay preferencia o su ruta ya no contiene un workspace, el directorio de
  datos predeterminado de Noteka.
- La ruta abierta se guarda como preferencia local; se reutiliza al siguiente
  arranque.
- La pantalla de bienvenida queda para el primer uso real (ningún workspace
  disponible), para errores de apertura y para Change workspace en Settings.
- Sin autocreación: el primer arranque conserva la creación explícita del
  workspace y su promesa de datos locales.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: apertura automática del workspace al iniciar.

## Impact

Svelte (`main.ts`, `App.svelte`, nuevo módulo de preferencia y arranque), un
puente de pruebas headless y documentación. Sin cambios Rust, SQLite, contratos,
operaciones, dependencias ni migraciones.
