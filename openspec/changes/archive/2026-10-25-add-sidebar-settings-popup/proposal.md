# Proposal

## Why

El pie del lateral expone controles y ruta del workspace permanentemente. El
usuario pide agruparlos en Settings y retirar el texto «En tu equipo» y el
punto azul de la marca, siguiendo sus dos capturas.

## What Changes

- Reemplazar el contenido del pie por un botón Settings con engranaje a la izquierda.
- Popup Settings con selector Tema, ruta del workspace y Cambiar workspace.
- Composición amplia inspirada en la captura de Notion aportada después:
  navegación lateral con Preferencias y General del workspace, contenido por
  secciones y búsqueda local de los ajustes existentes, preparada para crecer.
- Conservar aplicación/persistencia del tema, errores recuperables y guardias
  de todas las pestañas al cambiar workspace.
- Cerrar el popup con botón/Escape y devolver el foco; bloquear atajos de
  navegación detrás del modal y mantener el contenido usable en ancho estrecho.
- Mostrar sólo categorías y controles disponibles; no copiar funciones futuras
  o de cuenta de la referencia como si existieran en Noteka.
- Retirar el indicador azul y el texto solicitado, sin añadir ajustes nuevos.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: acceso y comportamiento del popup Settings desde el lateral.

## Impact

App/Svelte, CSS, un icono estático del catálogo instalado y adaptación de los
selectores existentes de revisión UI al nuevo acceso. Sin nuevas dependencias,
reglas Rust, contratos, migración ni operaciones. Los datos no se modifican al
abrir/cerrar Settings ni al cambiar tema. Revisión headless con workspace sintético;
no se usa pantalla ni workspace personal. No se crean suites nuevas.
