# Spec Delta

## ADDED Requirements

### Requirement: Movimiento fluido sin cambiar el ritmo aprobado

Los overlays SHALL conservar 400 ms de entrada y 200 ms de salida, geometría, contenido progresivo, vidrio y operaciones. Su reproducción MUST evitar mediciones de layout por frame cuando el anclaje y tamaño permanecen estables. La portada MUST conservar velocidad, escenas y pausas existentes con trabajo de diagnóstico acotado.

#### Scenario: Reproducción e interrupción
- **WHEN** se abren, cierran o interrumpen diálogos/menús mientras Home sigue visible
- **THEN** material y texto mantienen sus trayectorias y duración sin medir geometría en cada tick, con cierre inerte y limpieza de efectos

### Requirement: Gestos finos sin cancelación global

La clasificación de rueda SHALL permitir scroll nativo de gestos finos, horizontal y zoom sin un listener global cancelable permanente. El suavizado optativo de rueda discreta MUST limitar la intercepción al scroller correspondiente, conservar bordes anidados e interrupciones y liberar listeners al desactivar o desmontar.

#### Scenario: Cambiar de rueda a gesto fino
- **WHEN** se usa rueda discreta y luego trackpad, teclado, otro scroller o se cambia la preferencia
- **THEN** se conserva la operación apropiada, se libera la intercepción anterior y los gestos nativos no quedan retenidos por el suavizado

### Requirement: Side peek sin trabajo redundante de apertura

Side peek SHALL conservar 260 ms, anchos, foco y protección de borradores. El deslizamiento MUST usar transform y el canvas MUST evitar reconstrucciones de buffers por cada frame de resize. El cuerpo fresco de tareas/notas MUST quedar editable sin esperar relaciones o recursos secundarios; esos resultados MUST respetar la selección y generación vigentes. Las pistas de filas SHALL evitar precargas duplicadas sin reemplazar la lectura fresca del original.

#### Scenario: Original listo y metadatos demorados
- **WHEN** se abre una tarea/nota desde una fila y la lectura de relaciones sigue pendiente
- **THEN** el cuerpo original ya permite edición y cierre con protección de borrador, no se obtiene dos veces por la misma apertura y una respuesta tardía no reemplaza otro documento

#### Scenario: Apertura junto a la portada
- **WHEN** el panel reduce progresivamente el ancho de Home
- **THEN** la portada sigue animándose con sus buffers existentes, reconstruye al estabilizar las dimensiones y conserva nitidez, tamaño final y escena
