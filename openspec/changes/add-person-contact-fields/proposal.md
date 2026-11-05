# Proposal

## Why

El detalle Person inspeccionado en Notion permite editar título profesional, ubicación, teléfono, correo, perfiles, sitio e intereses. Noteka conserva empresa, etiquetas, cumpleaños y check-ins pero no ofrece esos campos locales.

## What Changes

- Agregar campos opcionales de contacto e intereses a Person, editables y validados en Rust, compartidos por escritorio, CLI y MCP.
- Mantener los datos locales, el guardado explícito, la recuperación y las propiedades desconocidas; no copiar valores de Notion ni activar servicios externos.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `knowledge-entities`: metadata opcional y validaciones de contacto local.
- `desktop-shell`: edición de las propiedades dentro de View details de Person.

## Impact

Payload JSON y contratos aditivos, validación Rust, interfaz y pruebas sintéticas. Sin migración SQL, importación de datos personales, enlaces automáticos ni proveedor remoto. Sin sync ni archivo.
