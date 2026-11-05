# Design

## Context

La pantalla de bienvenida (`App.svelte`, bloque `{#if !workspace}`) pide una
ruta y ejecuta `workspace.create` u `workspace.open`. `workspace` nace en `null`
en cada montaje, así que la aplicación siempre empieza ahí. El núcleo ya resuelve
el directorio de datos cuando la ruta se omite (`workspace::default_path`) y
devuelve `not_found` cuando la ruta no contiene `noteka.sqlite`.

Las preferencias de interfaz (tema, paneles, rueda, color) viven en
`localStorage` con módulos puros y claves versionadas. El arranque no tenía
preferencia equivalente.

## Goals / Non-Goals

**Goals:** abrir sin intervención un workspace existente al iniciar; conservar
la selección explícita cuando no hay ninguno o falla; no mostrar la bienvenida
como paso fijo; no cambiar el núcleo ni los contratos.

**Non-Goals:** autocrear un workspace vacío, migrar preferencias a SQLite,
recordar pestañas abiertas, cambiar la pantalla de bienvenida o los botones, y
abrir varios workspaces a la vez.

## Decisions

1. **Preferencia en `localStorage`.** Clave `noteka.workspace-path.v1`, con
   lectura/escritura tolerante a fallos como `theme.ts` y `panel-widths.ts`.
   Evita tocar contratos y mantiene la ruta fuera de SQLite.
2. **Arranque antes de montar.** `main.ts` resuelve el workspace y monta `App`
   con `startupWorkspace` y `startupError`. Así la bienvenida no parpadea en
   cada arranque y no hace falta un estado visual nuevo.
3. **Orden de intentos.** Último workspace guardado → directorio de datos
   predeterminado. Si el guardado falla con `not_found`, se intenta el
   predeterminado; si falla por otra causa (formato futuro, permisos), se
   muestra el error y la bienvenida sin abrir otro workspace como propio.
4. **Primer uso sin error.** Si ambos intentos devuelven `not_found`, se muestra
   la bienvenida sin mensaje: es el primer arranque, no una falla.
5. **Sin autocreación.** Crear datos personales sin una acción explícita
   contradice la promesa de workspace explícito; la creación sigue siendo un
   clic. La preferencia se guarda al crear o abrir correctamente, también desde
   la bienvenida.
6. **Puente headless.** El transporte de pruebas responde `not_found` a una
   operación sin `workspace`, igual que el núcleo con una ruta sin base; así los
   tests existentes conservan la bienvenida y no dependen del ordenador del
   desarrollador.

## Risks / Trade-offs

- Un workspace guardado que se movió cae al predeterminado; si tampoco existe,
  aparece la bienvenida. No se borra ni se mueve contenido.
- La ruta se guarda sólo tras abrir con éxito; una preferencia vieja no impide
  mostrar el error.
- El arranque agrega una invocación IPC antes del primer render; en un workspace
  grande la apertura (migraciones) puede demorar y la ventana queda sin pintar
  hasta montar. Se acepta porque es trabajo que el usuario igual debe esperar.
