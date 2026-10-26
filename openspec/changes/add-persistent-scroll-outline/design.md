# Design

## Context

Ver proposal.md. Home tiene seis secciones con data-home-section y h2 canónicos.
WorkspacePane mantiene un scroller propio y BlockEditor genera encabezados con
data-id. Las pestañas permanecen montadas e inactivas/ocultas, conservando scroll.

## Goals / Non-Goals

**Goals:** componente visual compartido, geometría del scroller real, estabilidad
durante cambios de contenido y acceso local sin tocar dominio/persistencia.

**Non-Goals:** tabla de contenido persistida, índices, nuevos tipos de bloque,
apertura de otra página, scroll global del documento o expansión a otros listados.

## Decisions

1. ScrollOutline se monta como hermano del scroller dentro del panel posicionado,
   fuera del contenido desplazable; evita un fixed sujeto a animaciones del tab.
2. Home usa sólo h2 de sus seis cabeceras. Note usa el título de página y H1/H2/H3
   con data-id del editor; títulos de vistas incrustadas/herramientas quedan fuera.
   El título permite orientación incluso en un documento vacío o no editable.
3. Se cachean nodos, etiquetas/niveles y posiciones en coordenadas del scroller.
   Scroll agrupa trabajo con RAF y busca la sección activa sobre posiciones ya
   medidas. ResizeObserver y MutationObserver actualizan cache por cambios reales;
   no hay polling, lectura de SQLite, parsing de JSON o LLM.
4. El primer tramo anterior a un título usa la primera marca y el final de un
   panel desplazable activa la última. Clic calcula scrollTop local con margen;
   el movimiento reducido usa salto inmediato. No cambia la selección del editor.
5. La marca activa es más clara y larga; H1/H2/H3 tienen largos discretos. Tooltip
   y aria-label conservan el nombre completo y aria-current identifica posición.
   El rail admite scroll interno para documentos grandes y reserva margen del
   contenido/scrollbar; no se oculta por hover ni se duplica en pestañas inactivas.
6. Seguimiento visual del 26/10: rail de 28px; marcas de 22px para título/Home y
   19/16/13px para H1/H2/H3, grosor 2px y centros separados 16px con puntero fino.
   La activa crece 2px, sin brillo; las demás usan un tono más tenue. En puntero
   grueso se conservan áreas de interacción de 24px. Clic, foco y scroll mantienen
   sus operaciones anteriores.

## Risks / Trade-offs

- Layout asíncrono → observadores de contenido/scrollport y recalculo al activar.
- Observer que se observa a sí mismo → rail fuera del árbol de contenido observado.
- Muchos encabezados → cache y actualización por frame, rail con altura acotada.
- Cambios de selección/scroll de otros tabs → observers sólo activos, cierre de
  listeners/RAF al ocultar/desmontar; no scrollIntoView de elementos internos.
- Encabezados iguales o texto largo → identidad por elemento, nombres escapados y
  tooltip acotado; no se escriben IDs/atributos dentro del documento.

## Migration Plan

Sin migraciones, contratos ni dependencias. Revertir la presentación no afecta
documentos. La verificación usa corpus sintético y navegador headless propio;
no se inspecciona ni modifica un workspace personal ni pantalla nativa.
