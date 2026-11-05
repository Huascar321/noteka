## ADDED Requirements

### Requirement: Jelly uniforme en accesos laterales

Home, Tasks, Notes, Projects, Tags, Goals, People, Search y Collections SHALL usar
el mismo rebote Jelly de Tasks, con iguales keyframes, intensidad y duración de
1000ms. MUST conservar figuras/colores, reposo exacto, clic explícito y limpieza
a1050ms, carga diferida, cancelaciones y reduced motion existentes.

#### Scenario: Clic en cualquiera de los nueve accesos
- **WHEN** se activa por ratón o teclado uno de los nueve accesos laterales habilitados
- **THEN** su figura reproduce el mismo Jelly, conservando el movimiento de Tasks, y vuelve a su SVG estático exacto

#### Scenario: Reposo y cancelación conservados
- **WHEN** sólo hay render/hover, se solicita movimiento reducido, la página se oculta o el acceso se desmonta durante carga/reproducción
- **THEN** no comienza una animación involuntaria ni queda reproducción diferida o estilo temporal activo
