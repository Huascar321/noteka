# Design

## Context

La referencia muestra Title, Location, Phone, Email, Instagram, LinkedIn, Twitter/X y Website; Interests es multiselección. Campos calculados y reuniones no se modelan por inferencia. PersonFields usa payload JSON opcional validado por Rust y CAS.

## Goals / Non-Goals

**Goals:** permitir capturar y recuperar datos elegidos por la persona en modo offline, manteniendo interoperabilidad y cambios explícitos.

**Non-Goals:** importar valores personales, enviar información a proveedores, hacer validación de existencia de cuentas, ejecutar enlaces o inferir campos calculados.

## Decisions

- Agregar textos opcionales acotados `job_title`, `location`, `phone`, `email`, `instagram`, `linkedin`, `twitter`, `website` y lista opcional `interests`. Ausencia y null se normalizan como en los campos existentes; sin modificación SQL ni migración de datos.
- Validar longitudes y controles en Rust; intereses siguen las mismas cuotas y unicidad de etiquetas de relaciones. No interpretar los textos como URLs confiables ni enviar nada.
- `home.query` omite los nuevos datos de contacto en la proyección de galería; `entity.get` los entrega al abrir el detalle y a clientes explícitos.
- Exponerlos en View details como texto e intereses individuales, sin partir etiquetas que contengan comas, preservando contenido previo y guardias. Valores futuros no interpretables siguen protegidos por la validación de fusión.
- Clientes anteriores con esquemas cerrados deben actualizarse antes de editar un Person con campos nuevos. Pruebas sintéticas cubren CAS, límites, campos desconocidos y reapertura.

## Risks / Trade-offs

- [Sensibilidad] → no leer/importar valores remotos ni guardarlos como fixtures; sólo datos sintéticos locales en pruebas y sin solicitudes externas.
- [Clientes antiguos] → contrato aditivo y limitación explícita; sin modificar formato ni datos existentes.
