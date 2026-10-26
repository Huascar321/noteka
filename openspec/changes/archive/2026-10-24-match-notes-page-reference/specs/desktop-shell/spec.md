# Spec Delta

## ADDED Requirements

### Requirement: Página Notes del menú lateral

Notes SHALL abrir una página amplia con icono propio sobre el título, navegación Inbox/Fav./All/Tags, New Note y vistas Notes/Inbox/Fav./Clips/Voice/Journal/Meetings/All. MUST iniciar Notes, mostrar filas con relaciones, tipo y antigüedad reales y mantener breadcrumb Home → Notes, captura/apertura protegidas y acciones textuales de ancho completo. All conserva acceso a todas las notas activas.

#### Scenario: Composición y vistas locales
- **WHEN** se abre Notes desde el menú lateral y se alternan sus vistas
- **THEN** aparece Notes inicialmente y se muestran sólo resultados de la consulta vigente, con iconos y relaciones locales, sin listado técnico paralelo ni recursos remotos

#### Scenario: Paginación y captura
- **WHEN** se activa Load more o New page desde el extremo derecho de su fila
- **THEN** se acumulan notas únicas con el cursor existente o se prepara un borrador protegido sin guardarlo; las categorías de creación conservan su tipo/favorito/inbox y los botones con borde mantienen su tamaño

#### Scenario: Actualización y salida
- **WHEN** se actualiza mediante ⌘R/Ctrl+R/F5 o se abandona la página durante una lectura
- **THEN** se conservan las vistas sin recargar el WebView, un diálogo/bloqueo impide iniciar la actualización y las respuestas de una instancia anterior no reemplazan la pantalla actual ni sus borradores

### Requirement: Tags integrado en la página Notes

La página Notes SHALL presentar Tags debajo, con Recent inicial y vistas Fav./A-Z/Types. MUST mostrar filas con icono, última nota navegable y clasificación disponibles, mantener lecturas/páginas independientes de Notes y conservar acceso al listado general de Tags, captura protegida, errores, temas y ancho estrecho. La cobertura permanece en Opciones de vista y las filas textuales de creación/paginación ocupan el ancho disponible.

#### Scenario: Tags con contexto real
- **WHEN** se abre Notes y se cambia la vista de Tags
- **THEN** Recent se presenta inicialmente y cada vista usa los datos locales existentes; última nota y clasificación se muestran sólo si están disponibles, sin modificar la vista elegida de Notes

#### Scenario: Errores y ancho estrecho
- **WHEN** falla una sección o se reduce el ancho en cualquiera de los temas
- **THEN** la sección afectada ofrece reintento sin simular una lista vacía, la otra conserva sus datos y los títulos/controles siguen accesibles sin overflow global
