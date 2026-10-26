# Design

## Context

Ver `proposal.md`. App posee tema/preferencia, workspace y guardias de cierre
agregadas. El lateral muestra esos controles al pie. SearchDialog ya utiliza
dialog nativo, foco/restauración y estado de apertura local; se reutiliza el patrón.
Durante la implementación el usuario añadió una referencia de Notion y pidió
una estructura que admita más ajustes, reemplazando la composición simple inicial.

## Goals / Non-Goals

**Goals:** agrupar los controles existentes y simplificar el lateral manteniendo
sus funciones y protección de datos.
**Non-Goals:** nuevos ajustes, página propia, datos de configuración en SQLite,
modelos/proveedores, cambios del núcleo o aceptación nativa nueva.

## Decisions

- SettingsDialog presenta props del tema/ruta/error y callbacks hacia App;
  no duplica estado persistente ni operaciones. App sigue aplicando saveTheme.
- Dialog nativo modal para inert/Escape, ciclo Tab explícito, botón de cierre
  visible y restauración del foco. En ancho estrecho el lateral conserva su estado bajo el
  modal para que el botón que lo abrió siga disponible al volver.
- La acción existente leaveWorkspace cierra Settings antes de preparar todas
  las pestañas. Su flujo de guardar/descartar/cancelar y errores se conserva;
  no se sustituye por limpiar tabs desde el popup.
- Bloquear atajos globales de pestañas/búsqueda detrás del modal, conservando
  controles de tema y cierre dentro. Montar el popup no remonta los paneles.
- Engranaje SVG estático seleccionado del catálogo AnimateIcons/Lucide ya
  instalado, mediante su generador; sin dependencia nueva ni animación lateral.
- Panel amplio de dos columnas: navegación Personal/Preferencias y
  Workspace/General; área de contenido con encabezados, grupos y filas de ajuste.
  En ancho estrecho, navegación superior y contenido con scroll independiente.
- Metadatos locales de secciones/ajustes alimentan un filtro de texto, sin
  lecturas del workspace. Mostrar únicamente los tres controles reales y vacío
  con limpieza; no copiar secciones de cuentas, pagos o funciones futuras de Notion.
- Adaptar sólo los accesos de selectores UI existentes al nuevo botón. Revisar
  el producto compilado con CLI/SQLite sintéticos, sin suites nuevas ni pantalla.

## Risks / Trade-offs

- Dos diálogos superpuestos al cambiar workspace → desmontar Settings antes
  de prepareTabs, conservando su guardia y el foco de la pestaña pendiente.
- Atajos activos detrás del modal → bloquearlos explícitamente en App.
- Ruta larga o viewport bajo → ancho acotado, texto con wrap y scroll interno.

## Migration Plan

Sin migración ni cambios de contratos/datos. Revertir los archivos de UI y
catálogo restaura el pie anterior; preferencias y workspace siguen intactos.
