# Verificación: apertura automática del workspace

Fecha: 2026-11-05. Alcance: `auto-open-workspace` (UI y arranque; sin cambios
Rust, SQLite ni contratos).

## Ejecutado

| Comando | Resultado |
| --- | --- |
| `npm run test:startup` | 7/7 pruebas: preferencia, tolerancia a storage roto, último → predeterminado, primer uso silencioso y error distinto de `not_found`. |
| `npm run check` | 0 errores, 0 warnings. |
| `npm run build` | Iconos, contratos y bundle OK. El aviso de chunk > 500 kB es previo. |
| `npm run test:headless-startup` | 7 checks: primer arranque pide selección, la ruta se guarda al crear, un reinicio reabre sin pedir ruta, vuelve a la selección al borrar la preferencia, y una ruta inválida no presenta otro workspace. Sin errores de página. |

Reinicio real con `npm run tauri dev`:

1. Primer arranque sin preferencia: abrió el workspace del directorio de datos
   directamente en Home y escribió `noteka.workspace-path.v1` en el
   `localStorage` de WebKit (`~/Library/WebKit/app.noteka.desktop/...`).
2. Segundo arranque: Home directo, sin pantalla `#workspace-path`.

Capturas fuera de Git: `/tmp/noteka-autoopen.png` (primer arranque con Home) y
`/tmp/noteka-reopen.png` (reapertura desde la preferencia). No se inspeccionó el
`localStorage` del WebView con DevTools; se leyó la clave persistida con
`sqlite3` sobre `localstorage.sqlite3` antes del segundo arranque.

## Límite conocido de la suite headless histórica

`npm run test:headless` sigue fallando con `Timeout: Home vacío`. La causa es
previa a este cambio y no está en el arranque: `tests/native/home.js` espera
`.home-coverage` visible al abrir Home, pero ese nodo existe sólo dentro de
**View options** (`{#if optionsOpen}`, `optionsOpen = $state(false)`) tanto en
`HEAD` como en el árbol actual.

Evidencia:

```sh
git show HEAD:src/lib/HomeTaskSection.svelte | grep -c '{#if optionsOpen}.*home-coverage'   # 1
git show HEAD:tests/native/home.js | grep -n 'home-coverage'                                # lines 6, 64, 65
```

`tests/headless-navigation.cjs` usa la misma condición
(`document.querySelectorAll('.home-coverage').length===5`), así que tampoco
puede pasar hoy. No se modificó esa presentación en este cambio ni se dan por
aprobadas esas suites: quedan rojas preexistentes por esa discrepancia.

Ajustes de tests incluidos, aún no validados por las suites rojas: el puente
headless responde `not_found` a una operación sin `workspace` (en vez de 500),
`tests/headless-run.cjs` neutraliza sólo la preferencia de workspace para
conservar el recorrido por la bienvenida y `tests/headless-evidence.cjs` la
borra antes de su recarga. `tests/headless-navigation.cjs` agrega un check de
reapertura que se ejecutará cuando esa suite deje de fallar por la condición
anterior.

## No ejecutado

- `npm run test:native` (WKWebView nativo) y la suite Rust: este cambio no toca
  Rust ni SQLite.
- Prueba en un perfil de usuario sin workspace en el directorio de datos: el
  primer uso se cubre con los tests unitarios y el transporte headless (puente
  responde `not_found`), no con un perfil real borrado.

## OpenSpec

`npx --yes @fission-ai/openspec@1.14.1 validate auto-open-workspace --strict`
pasó antes de archivar, y el cambio quedó en
`openspec/changes/archive/2026-11-05-auto-open-workspace` con un requisito
nuevo (378 caracteres) en `desktop-shell`. `spec/desktop-shell` continúa
fallando la validación estricta por 16 requisitos de más de 500 caracteres que
ya existían en `HEAD` (86 requisitos, 16 largos); este cambio no agrega ninguno
(87 requisitos, 16 largos). Los otros cinco ítems rojos de `validate --all
--strict` son cambios activos previos.
