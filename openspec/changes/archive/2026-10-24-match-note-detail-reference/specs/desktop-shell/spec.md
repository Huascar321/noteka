# Spec Delta

## ADDED Requirements

### Requirement: Detalle de Note con propiedades locales y cuerpo protagonista

Una Note normal SHALL presentar icono y título editables, Tag/URL/Favorite/Type compactos y relaciones locales antes del cuerpo. Tipo y favorito MUST editar el payload local; Tag y las relaciones MUST abrir los originales con guardias existentes. URL importada SHALL identificarse como lectura. Inbox, procedencia y snapshot completo SHALL quedar accesibles en View details sin duplicar snapshots como controles vigentes. Goal/Milestone y Task MUST conservar su composición propia, sin migraciones ni writes al abrir.

#### Scenario: Referencia importada y edición local
- **WHEN** se abre una Note importada y se cambia Type o Favorite
- **THEN** los controles representan el payload local pendiente, el snapshot original sigue en View details y navegar a un tag o proyecto exige resolver Guardar/Descartar/Cancelar

#### Scenario: Ausencias y variantes
- **WHEN** falta URL o una relación o se abre Goal/Milestone, Task o formato desconocido
- **THEN** la Note muestra ausencia real sin inventar destinos; las variantes conservan sus controles y los documentos desconocidos siguen recuperables sin conversión

#### Scenario: Temas y ancho estrecho
- **WHEN** se abre la Note en cualquiera de los temas o a 420 px
- **THEN** título, propiedades, relaciones y cuerpo permanecen accesibles sin overflow global y los bloques de código conservan scroll propio

#### Scenario: Propiedades repetidas del export
- **WHEN** una Note acreditada de la base Notes contiene el prefijo reconocido y un primer párrafo plano de metadata con al menos tres coincidencias exactas y sólo nombres admitidos
- **THEN** el párrafo se presenta en View details y se recompone íntegro al editar; nombres desconocidos, valores contradictorios, marcas o texto ambiguo permanecen en el cuerpo
