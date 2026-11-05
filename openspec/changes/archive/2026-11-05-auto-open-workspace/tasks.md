# Tasks

## 1. Arranque

- [x] 1.1 Crear la preferencia de ruta y el arranque automático con intentos último → predeterminado, y agregar tests unitarios del orden de intentos y de la preferencia.
- [x] 1.2 Resolver el workspace antes de montar la interfaz y conectar la ruta inicial y el error de arranque en `App.svelte`; guardar la ruta al crear o abrir.
- [x] 1.3 Ajustar el puente headless para responder `not_found` a una operación sin `workspace`.

## 2. Documentación y verificación

- [x] 2.1 Documentar el arranque en `docs/desktop-navigation.md` y registrar la evidencia en `docs/verification-auto-open-workspace.md`.
- [x] 2.2 Verificar con build/check, tests unitarios, un test headless y un reinicio real de la aplicación.

## Verification

`npm run test:startup` 7/7, `npm run check` sin diagnósticos, `npm run build` OK
y `npm run test:headless-startup` 7/7 con reinicio real en dos arranques. La
suite `test:headless` sigue roja por `.home-coverage` oculto en View options,
condición previa presente en `HEAD`; detalles en
[`docs/verification-auto-open-workspace.md`](../../../docs/verification-auto-open-workspace.md).
