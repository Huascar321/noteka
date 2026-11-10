# Design

## Context

Ver `proposal.md`. Search hereda el material de Settings: blur 24 px, tinte claro 42% y velo blanco 20%. La transición copia el material calculado del dueño; debe conservar un solo filtro local.

## Goals / Non-Goals

**Goals:** aproximar transmisión, tinte gris y contorno de la referencia sin perder lectura ni comportamiento.

**Non-Goals:** refracción nativa, transparencia al escritorio, rediseño de layout, modificación de Settings o del núcleo.

## Decisions

- Variante local de `.context-search.glass-popup` con tokens propios para tinte, sheen, filtro y velo. Settings conserva sus valores. Se descarta cambiar globalmente `.glass-popup` porque alteraría otros diálogos no pedidos.
- Desenfoque corto y saturación contenida dejan reconocer el fondo; el marco compartido proporciona reflejos. Resultados, preview y pie no superponen otra película blanca. La tinta de Search se oscurece/aclara por tema para mantener contraste sobre extremos compuestos; el alfa final se decide con medición.
- Base sólida sin soporte de backdrop-filter y fallbacks posteriores de accesibilidad. Sin filtros hijos, animaciones decorativas nuevas ni dependencias.
- Rust/SQLite conservan propiedad de reglas y datos. No hay migración. Capturas sintéticas del frontend real con CLI/SQLite verifican web; no se atribuye composición nativa a esa evidencia.

## Risks / Trade-offs

- [Transparencia reduce contraste] → comprobar píxeles sobre extremos y tinta real; ajustar el tinte sin ocultar el fondo.
- [WebKit y Chromium componen distinto] → capturar ambos motores y declarar límite nativo.
- [Cambios previos en los mismos archivos] → editar sólo la receta Search y la sección correspondiente de la guía.

## Migration Plan

Aplicación local por build/HMR; reversión limitada a las nuevas reglas y documentación. No requiere despliegue, commit ni publicación.
