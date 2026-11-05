# Design

## Context

El estilo nativo varía por motor y actualmente no existe una receta global. La barra de ScrollOutline está oculta intencionalmente.

## Goals / Non-Goals

Barra fina, sobria y usable en ambos ejes y temas. No sustituir scroll nativo ni cambiar el contenido.

## Decisions

- WebKit/Blink: pista transparente, canal vertical de 9 px y horizontal de 8 px (seguimiento del 27/10/2026), tirador redondeado con borde transparente y tono tenue; color más claro al interactuar. Conservar la regla específica de ScrollOutline.
- Motores sin pseudo-elementos WebKit: propiedades estándar thin y colores de tema. Mantener contraste y preferencias de sistema.

## Risks / Trade-offs

Los motores pueden aplicar barras overlay o configuraciones del sistema de manera distinta; verificar estilo y desplazamiento sin declarar aceptación nativa no observada.

## Migration Plan

Sin cambios persistentes; revertir el CSS restaura la apariencia anterior.
