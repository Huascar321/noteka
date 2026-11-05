# Design

## Context

La maqueta propone reflejos de escritorio, pero Noteka usa un WebView en ventana opaca y el usuario prioriza rendimiento. Los originales, vistas, bordes, tamaño y scroll deben seguir utilizables.

## Goals / Non-Goals

Proponer sensación de vidrio en el marco sin sacrificar rendimiento, navegación, acento, foco, temas o accesibilidad. No prometer Liquid Glass nativo ni translucidez del escritorio y no rediseñar búsqueda global ni menús en este primer cambio.

## Decisions

Gradientes CSS estáticos de baja complejidad en las dos superficies del marco; bordes tenues y fondos de controles derivados del acento y tema. No agregar filtros de desenfoque, capas animadas, temporizadores ni cambios de transparencia de ventana. La jerarquía, geometría e instancias existentes se conservan. Respetar reducir transparencia con superficies sólidas y estilos de foco existentes.

Seguimiento: recuperar la base azul grisácea oscura de la maqueta en el contenido y un gradiente estático suave en Home. La portada ASCII transparente comparte el mismo fondo de Home, sin unión de colores; la portada fotográfica conserva su imagen. El tema claro conserva su fondo; no se agregan imágenes, filtros ni animaciones al contenido.

Refinar el sidebar con reflejos estáticos suaves en capas de gradiente que respondan al acento y al tema, conservando su contraste y superficie sólida cuando se solicita menor transparencia o mayor contraste. No capturar ni filtrar el escritorio.

La revisión final de `refine-neutral-glass` sustituye los tintes derivados del color primario por fondos neutros. El color primario sigue en foco, iconos y acciones; los colores de estados o contenido específico mantienen su significado propio.

## Risks / Trade-offs

La aproximación no refracta el escritorio como el material nativo. Validar colores claros/oscuros, contraste, vista estrecha, animación de sidebar y estabilidad; registrar que no se miden garantías absolutas de consumo físico.
