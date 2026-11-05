# Design

## Context

Ver motivación en proposal.md. AppIcon centraliza 85 claves; hay nueve animaciones
laterales cargadas a demanda. Las imágenes personales usan claves image-UUID.
Icona declara MIT en el sitio y en packages/icons/package.json; su revisión
dd9f5217720a3f459a1f5464125701eb0d51c3a2 exporta SVG/CSS sin React en ejecución.

## Goals / Non-Goals

**Goals:** integración local reproducible, peso Fill/bold, máscaras aisladas y
política de movimiento actual. **Non-Goals:** cambiar datos, permisos, marca o
controles nativos; animación constante o canvas/partículas.

## Decisions

- Fijar revisión upstream y guardar sólo exportaciones elegidas, declaración MIT
  y manifiesto. Un importador explícito contra un checkout de esa revisión usa el
  exportador oficial; el generador/build leen sólo recursos locales. No importar
  el paquete React ni depender de un npm no publicado.
- Conservar las 85 claves y seleccionar Fill cuando esté disponible, stroke 2.4
  en outline. Las figuras ausentes usan equivalentes documentados; users,
  user-plus y lock-open pueden adaptar/componer geometría oficial para conservar
  distinciones. No rellenar caminos outline arbitrariamente.
- AppIcon sigue siendo estático. Aislar IDs de máscaras por instancia. Mantener
  currentColor y ruta de imágenes personales con fallback.
- La continuación `unify-sidebar-jelly-motion` aplica la receta oficial Jelly de
  Tasks a las nueve figuras, con CSS compartido, sin efectos interactivos/partículas
  y duración acotada. Al pulsar, montar temporalmente escena y estilo aislados por
  instancia; ejecutar una iteración, restaurar nodos originales y eliminar estilo.
  Mantener cancelación global, nueva pulsación, reduced motion y visibility.
- Rust sigue siendo dueño de reglas y datos; no hay cambios de contrato,
  persistencia, migración ni recuperación de formato.

## Risks / Trade-offs

- Catálogo sin correspondencia exacta para cada concepto → equivalentes explícitos
  y revisión del catálogo completo a tamaños reales; conservar etiquetas.
- Colisiones de máscaras o estilos → IDs/classes/keyframes aislados por instancia.
- Movimiento o errores de carga → fallback estático, una iteración y limpieza.
- Upstream no incluye archivo LICENSE separado → conservar sus declaraciones MIT
  originales y texto MIT con atribución a Icona/Komal Verma, sin atribuir una copia
  inexistente al repositorio. Sin actualización automática.

## Migration Plan

Generar SVG/CSS locales, sustituir reproductor, retirar dependencia anterior y
comprobar build/catálogo/UI. Revertir estos archivos recupera el set anterior sin
transformar registros. La verificación será headless aislada, sin aceptación nativa.
