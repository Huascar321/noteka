# Design

## Context

La búsqueda usa un diálogo opaco y un desenfoque de todo el backdrop. Los menús de acciones, peek, relaciones y editor se montan cuando se usan. La portada ASCII puede seguir animándose mientras un modal cubre Home.

## Goals / Non-Goals

Dar vidrio coherente y legible en claro y oscuro, preservar comportamiento, foco y capas, y limitar la composición. No cambiar ventana nativa ni prometer equivalencia con Liquid Glass nativo.

## Decisions

Aplicar una superficie casi opaca con tinte y desenfoque pequeño de 6 px sólo en búsqueda y menús montados, cuando el WebView lo soporte. El backdrop completo se oscurece sin filtro. Ante transparencia reducida o contraste aumentado se usa fondo sólido sin filtro. Suspender el reloj de la portada cuando Home está bloqueado y reanudarlo sin reiniciar al cerrar. No introducir temporizadores ni animaciones nuevas.

Seguimiento solicitado: la primera versión quedó demasiado opaca respecto de la
maqueta. Aumentar transparencia y usar desenfoque de
10 px acotado al panel, con texto secundario ajustado para mantener contraste;
la ventana y el backdrop completo siguen sin filtros ni animaciones adicionales.

La revisión `refine-neutral-glass` usa gris neutro en estas superficies y reduce
la cobertura visual de la búsqueda, conservando el filtro local.

## Risks / Trade-offs

Desenfocar incluso un panel acotado tiene costo dependiente de contenido y GPU; se verifican actividad de portada y estilos, y se reporta cualquier medición y sus límites. Verificar composición y lectura sobre fondos variables y controles de teclado.
