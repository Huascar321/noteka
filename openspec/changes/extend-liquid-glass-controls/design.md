# Design

## Context

El sidebar aprobado usa tinte, gradientes y una máscara anular en dos pseudoelementos. Las pestañas superiores, breadcrumbs, indicador deslizante y New ya tienen selectores compartidos en `src/style.css`; sus operaciones y transiciones no necesitan cambios. Ver motivación en `proposal.md`.

## Goals / Non-Goals

**Goals:** compartir la dirección de luz y la curvatura del vidrio con jerarquía por función, conservando tamaño y comportamiento.

**Non-Goals:** transparentar la ventana, refracción nativa, rediseñar cuerpos de contenido o modificar datos/contratos/Rust.

## Decisions

- Compartir la receta de borde y relieve del sidebar mediante variables CSS locales. Las variantes neutras reducen opacidad de borde y sombra; el breadcrumb conserva filtro local de 3 px y un reflejo más tenue. Se evita copiar una receta completa por componente.
- La pestaña superior activa adopta radio de cápsula y mantiene 32 px de alto. Dar margen interno al contenedor de tabs para no cortar el brillo; +/× mantienen sus áreas y se redondean en hover/foco. La alternativa de filtrar toda la barra agregaría composición sin contenido útil.
- El indicador de vistas conserva su acción y movimiento. New conserva su clase compartida y texto blanco; la paleta visual deriva un tinte más oscuro con margen de contraste para los reflejos, sin alterar el acento de iconos o de otros botones. Se evita aclarar el texto por la elección explícita previa de blanco.
- Colocar los fallbacks después de las variantes, ocultar pseudoelementos decorativos y mantener foco, semántica y hit targets. Las capas no reciben eventos; no se agregan animaciones ni dependencias.
- Rust y SQLite conservan el contenido y sus operaciones; no cambia el formato persistente ni se requiere migración.

## Risks / Trade-offs

- [Reflejos sobre el tinte de New] → medir píxeles compuestos en reposo/hover y ambos temas/acento; ajustar sólo el tinte de esta acción.
- [Sombras recortadas] → revisar contenedor de tabs, vista activa y ventana estrecha.
- [Breadcrumb sobre contenido] → conservar fallback sólido; el acabado CSS no garantiza refracción ni contraste sobre todos los fondos en movimiento.

## Migration Plan

Aplicación local por HMR, sin publicación ni migración. El commit `affca8a` conserva el estado anterior al sidebar y el diff de este cambio queda local junto al sidebar aceptado. Los deltas permanecen activos hasta su cierre explícito.
