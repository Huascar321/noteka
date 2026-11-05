# Proposal

## Why

Home necesita clima útil para la ubicación actual, sin consultas remotas implícitas.

## What Changes

- Set location solicita directamente la ubicación actual con permiso del sistema; condiciones actuales y ocho fechas.
- Se elimina la selección manual de ciudad fija.
- Caché local, actualización periódica, estados offline y atribución a Open-Meteo.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: clima optativo en Home.

## Impact

Home, preferencias locales y CSP; consultas HTTPS a Open-Meteo sin credenciales ni cambios en SQLite. Uso alojado gratuito sujeto a condiciones no comerciales del proveedor.
