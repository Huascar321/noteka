# Proposal

## Why

El usuario acepta el vidrio gris transparente de Search y pide propagarlo a todos los popups que conservan el material anterior. Hoy Settings, guardias, menús y pickers siguen usando tintes densos, velos blancos o blur 10/24 px.

## What Changes

- Convertir la receta de Search en el material compartido de superficies flotantes: tinte gris neutro, sheen tenue, blur 6 px y tinta con contraste.
- Aplicarla a Settings, confirmaciones, Move project, menús, pickers, herramientas flotantes, popovers del editor, Due y tooltips que tenían vidrio.
- Retirar los velos opacos de Settings y excepciones antiguas de confirmación; mantener acciones semánticas, layout, anclaje, transición y fallbacks sólidos.
- Actualizar el design system como receta vigente general y registrar inventario, capturas y verificaciones.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: material transparente coherente de todos los popups que usaban vidrio anterior.

## Impact

CSS compartido, documentación y evidencia. Sin cambios de operaciones, datos, Rust, dependencias, ventana nativa ni controles del sistema. Contenido documental y navegación persistente conservan su categoría visual.
