# Identidad de Noteka

Identidad aprobada el 05/11/2026: una «n» azul de nota adhesiva con la esquina
superior derecha levantada, y el nombre Noteka en sans serif de peso 450.
La zona liberada por la esquina revela el fondo; la solapa clara muestra el
reverso del papel. No es un triángulo pintado dentro de un contorno completo.

## Fuentes de producción

- `src/lib/noteka-brand.json`: geometría y paleta compartidas.
- `src/lib/NotekaMark.svelte`: SVG de UI, estático, sin filtros o animaciones.
- `src-tauri/icons/Noteka.icon/`: documento editable de Icon Composer. Fondo
  neutro, cuerpo azul y solapa clara separados; el sistema aporta su material.
- `src-tauri/icons/Assets.car`: catálogo nativo compilado para macOS.
- `.icns`, `.ico` y PNG: recursos de compatibilidad. `public/noteka.svg` es el
  favicon local con tema del sistema.

Azul de marca `#3183a6` y azul oscuro `#56b1cd`; el nombre usa `--text`.
La marca mantiene su identidad cuando cambia el acento personal de los
controles. En contraste aumentado, el SVG de UI usa tinta del tema y borde de
papel definido. Los íconos de entidades personalizados no se modifican.

## Generación y comprobación

```sh
# Sólo al modificar geometría o material. Requiere macOS y Xcode con Icon Composer.
npm run brand:generate
# Sin herramientas de Apple; también se ejecuta como parte de npm run build.
npm run brand:check
# Capturas focales con el frontend compilado y un corpus SQLite sintético.
npm run build
npm run brand:screenshots
# Segundo motor opcional.
NOTEKA_TEST_BROWSER=webkit npm run brand:screenshots
```

El generador no usa red. Deriva los SVG y favicon desde la geometría; compila
con `actool`, exporta PNG/ICO con `ictool` y conserva `.icns` del compilador.
Los PNG se normalizan a RGBA de 8 bits con CoreGraphics/ImageIO para cumplir
los requisitos de Tauri y del ícono de Windows, conservando transparencia.
`brand-manifest.json` registra huellas de fuentes/salidas y versión de Xcode.
La comprobación detecta recursos desactualizados sin volver a compilar Apple.
Los renders de inspección Default/Dark/ClearLight/ClearDark quedan en
`output/playwright/brand/native/`. Clear se presenta monocromo según el sistema.

En macOS, el empaquetado incluye el catálogo y la referencia `CFBundleIconName`.
El `.app` temporal de `tauri dev` también contiene `Assets.car` y `icon.icns`
embebidos: un ejecutable trasladado no depende del checkout para encontrar el
arte. Se conservan identificador, permisos y firma local de desarrollo.

El efecto de Liquid Glass lo renderiza Apple según versión y apariencia. Las
láminas de la conversación eran simulaciones; los PNG generados por `ictool`
son el render nativo de producción. No se fuerza un cambio de apariencia del
sistema, se reinicia el Dock o se borran sus cachés para actualizar la marca.

## Referencias

[Icon Composer de Apple](https://developer.apple.com/icon-composer/) y
[guía de íconos](https://developer.apple.com/design/human-interface-guidelines/app-icons).
Ver `docs/verification-brand.md` para evidencia de esta implementación.
