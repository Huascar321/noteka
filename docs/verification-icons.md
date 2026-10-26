# Registro histórico — iconos y apariencia

Fecha: 23 de octubre de 2026.

Este registro corresponde al catálogo Lucide anterior. El catálogo vigente y su
revisión acotada están en [AnimateIcons](verification-static-animateicons.md);
las suites de este registro no se repitieron durante ese reemplazo.

## Catálogo

- Paquete: `@lucide/svelte` 1.61.0, licencia ISC.
- 24 iconos seleccionables; navegación y estados usan SVG, no emojis.
- Los iconos heredan `currentColor`; el color explícito se guarda como `#RRGGBB`.

## Comprobaciones

```text
npm run check                         ✓ 0 errores
npm run build                         ✓ contratos, check y Vite
cargo test -p noteka-core             ✓ 267 pasados, 16 ignorados
npm run test:headless                 ✓ 592 checks + 14 de teclado
NOTEKA_NATIVE_SCREEN=1 … test:native   ✓ recorrido nativo de 592 checks
```

Se probó en la UI real del workspace importado:

1. Abrir una nota con el selector de icono.
2. Elegir `Estrella` y color `#0b6e99`.
3. Pulsar `Listo` y `Guardar`.
4. Verificar que la fila de Notes y la cabecera de la página muestran el icono
   con color, y que SQLite conserva:
   `{"icon":"star","icon_color":"#0b6e99","inbox":false}`.
5. Recargar: el icono y el color siguen presentes.

También se verificó el rechazo del núcleo para una clave inválida (`Star!`) y se
comprobó que los payloads sin icono siguen serializándose sin los campos
opcionales.

## Límites

- El export Markdown de Notion no trae los iconos originales; las páginas
  importadas usan el icono por defecto del tipo.
- La captura nativa de Home se hizo a 900, 1280 y 1920 px sin overflow; el
  tiburón ASCII se escala para no cortarse en 900 px.
- El selector no ofrece carga de SVG ni imágenes externas. Para eso hace falta
  un cambio de producto separado con validación de archivos.
