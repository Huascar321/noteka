# Design

## Context

Ver `proposal.md`. Los centros actuales se sitúan todos entre el 40 % y el 54 %
del ancho; sus radios permiten coincidir en casi cualquier zona. El nado
articulado, reloj, renderer y paletas ya están aprobados.

## Goals / Non-Goals

**Goals:** dispersión sostenida y posiciones variadas con el mismo nado suave.
**Non-Goals:** impedir cada cruce, física de colisiones, filas rígidas o modificar
geometría/apariencia/ritmos/tamaño. Sin datos, operaciones o migración.

## Decisions

- Configuración explícita por recorrido: centros de tiburones a izquierda y
  derecha, radios horizontales que mantienen separación; pequeños con centros
  altos/bajos y fases distintas. Cada animal conserva un área de movimiento
  amplia, dirección y ritmo propios, sin saltos ni cambios de carril.
- Mantener las elipses suavizadas y su orientación/curvatura analíticas. Se
  descarta repulsión aplicada sólo a posiciones porque desalinearía el cuerpo
  respecto del movimiento; un simulador de colisiones añade estado innecesario.
- Revisión comparativa de centros durante treinta minutos simulados por tamaño
  y capturas del renderer real a varios tiempos, temas y ancho estrecho. El
  indicador de cercanía de centros no equivale a intersección de siluetas.
- Cambiar sólo presentación local en TypeScript. SQLite/originales y operaciones
  Rust siguen siendo autoridad, sin acceso ni formato persistente nuevo.

## Risks / Trade-offs

- Regiones demasiado pequeñas → conservar variación horizontal y vertical
  observable, inspeccionar secuencias y movimiento, sin inmovilizar a los peces.
- Siluetas grandes en una portada baja → permitir recortes/cruces ocasionales
  naturales; comprobar que desaparece la acumulación de casi todo el elenco.

## Migration Plan

Sin migración. Revertir sólo los parámetros del recorrido recupera la composición
anterior; no afecta datos ni preferencia de portada.
