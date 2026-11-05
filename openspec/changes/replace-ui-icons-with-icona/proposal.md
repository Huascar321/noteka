# Proposal

## Why

El usuario pide reemplazar el set de iconos por Icona.works porque quiere figuras
rellenas o bold y conservar soporte de animaciones. El catálogo vigente usa
AnimateIcons y trazos finos.

## What Changes

- Sustituir los 85 nombres lógicos por SVG de Icona: Fill cuando exista, trazo
  bold en los demás controles y equivalentes semánticos cuando falte la figura.
- Conservar tamaños, colores, etiquetas, claves persistidas e imágenes personales.
- Reemplazar el reproductor lateral por las animaciones CSS oficiales exportadas
  por Icona, con un ciclo por clic y las cancelaciones actuales.
- Fijar revisión, recursos, procedencia y declaración MIT, sin dependencias React,
  CDN o red en ejecución. Retirar la dependencia AnimateIcons.

Los controles nativos de ventana y la marca de la app conservan su identidad.
No cambia dominio, SQLite, documentos ni operaciones públicas. No agrega
animaciones por hover o reproducción continua.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: catálogo Icona local Fill/bold y reproducción CSS lateral.

## Impact

`AppIcon`, catálogo/generador, reproductor lateral, toggle de sidebar, licencias,
package/lock y documentación de iconografía. Se verificará generación reproducible,
build, todos los glifos en ambos temas y movimiento/cancelación en navegador
headless aislado. La inspección headless no acredita aceptación nativa.
