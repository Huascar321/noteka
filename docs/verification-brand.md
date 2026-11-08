# Verificación de identidad de Noteka

Fecha: 05/11/2026. Cambio activo `adopt-peeled-note-identity`.

## Alcance implementado

- Geometría vectorial compartida de «n» con esquina vacía al exterior y solapa
  clara independiente; azul de marca, nombre de peso 450 y fallback de contraste.
- Bienvenida/sidebar y favicon locales. No se agregan acciones, fuentes remotas,
  filtros, animaciones, dependencias o migraciones.
- Documento editable de Icon Composer, catálogo nativo macOS y PNG/ICNS/ICO.
- Recursos embebidos para el `.app` temporal de desarrollo. Mismo identificador
  y permisos de ubicación; el arte no depende de una ruta al checkout en runtime.

## Evidencia

- `npm run brand:generate`: `actool` compiló el catálogo y `ictool` produjo
  renders Default/Dark/ClearLight/ClearDark con Xcode 27.2 en macOS 27.2.
- `npm run brand:check`: huellas de cuatro fuentes y nueve salidas; tamaño y
  formato RGBA de 8 bits de los PNG y los seis frames del ICO.
- `npm run build`: correcto, `svelte-check` con 0 errores/0 advertencias. Vite
  conserva una advertencia por el tamaño del chunk principal; no se hizo una
  optimización de bundles en este cambio.
- `npm run brand:screenshots` y `NOTEKA_TEST_BROWSER=webkit npm run
  brand:screenshots`: 20 comprobaciones por motor, 10 capturas por motor.
  Incluyen ambos temas, marca/nombre, foco decorativo, navegación a Notes,
  apertura/cierre de Search, sidebar compacto, contraste aumentado y 420 px.
  Cero errores de página y cero solicitudes externas.
- `cargo test --offline --locked -p noteka-desktop --bin noteka-desktop
  dev_bundle::tests -- --nocapture`: 1/1. El paquete temporal tiene los bytes
  correctos de catálogo/ICNS y las referencias de Info.plist.
- Build local macOS con `npm run tauri -- build --debug --bundles app --config
  '{"bundle":{"active":true}}' -- --offline --locked`: `.app` creado sin
  abrirlo. Recursos y `CFBundleIconName=Noteka` contrastados contra las fuentes;
  identificador `app.noteka.desktop` y permiso de ubicación preservados.
- El `.app` local se firmó ad hoc con `codesign --force --deep --sign -` y pasó
  `codesign --verify --deep --strict`. Es una firma de desarrollo local, sin
  certificado de distribución o notarización.
- `openspec validate adopt-peeled-note-identity --strict`, comprobaciones de
  sintaxis JS, formato Rust y `git diff --check` pertinentes.

La [galería](design/brand/README.md) conserva renders nativos y capturas reales
del frontend. Los manifiestos registran huellas/procedencia; no hay notas,
contactos o rutas de workspaces personales en los PNG.

## Incidencias corregidas

El exportador Apple emitió algunos PNG de 16 bits. Se agregó normalización con
CoreGraphics/ImageIO y validación del formato real, conservando alfa y evitando
esa incompatibilidad con Tauri/Windows. El primer recorrido de capturas tenía
un selector Notes ambiguo; se acotó a la navegación, sin cambios de producto.
El paquete debug inicial no tenía una firma de bundle válida para verificar
recursos; la firma ad hoc del artefacto local final corrigió esa comprobación.
No se cambió la política de firma de distribución del proyecto.

## Límites

La app local no se abrió sobre un workspace personal ni se verificó el Dock
en pantalla. Los renders nativos proceden de `ictool`; la inclusión del catálogo
y su metadata se verificó en el paquete. Los recorridos de UI usaron transporte
headless, no IPC nativo. No se ejecutaron las suites generales del dominio,
Windows ni publicación/notarización. Los íconos de compatibilidad están
generados y validados como archivos; eso no acredita ejecución en Windows.

El material dinámico varía según macOS/apariencia. Una app que ya estaba abierta
requiere reconstrucción/reinicio para incorporar los recursos nuevos; no se
reinició Dock ni se eliminaron cachés globales.

El cambio queda activo, sin sincronizar specs principales ni archivar. No se
realizó commit, push o publicación.
