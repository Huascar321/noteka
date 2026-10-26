# Design

## Context

Ver `proposal.md`. `App.svelte` concentra workspace, menú, lecturas, editor y guardias en una sola instancia; `sections` también resuelve títulos. Home tiene su propio catálogo y no sustituye My Day/Overdue. Search llama a `search.query` y `extraction.search`, y la búsqueda agrupada mantiene resultados independientes. El checkout contiene muchos cambios previos que deben conservarse.

La referencia real se inspeccionó con la sesión Playwright existente `notion-whatsapp`, cuyo perfil coincide con el solicitado. La barra mide unos 32 px; el diálogo tiene ancho máximo cercano a 1000 px, altura 700 px, esquinas de 20 px, sombra suave, resultados a la izquierda y preview a la derecha. La portada se obtuvo como PNG original autorizado. Capturas/URLs de cuenta se conservan sólo fuera de Git o en salida ignorada.

## Goals / Non-Goals

**Goals:** una navegación compacta y un único buscador, independencia de edición entre pestañas y composición visual fiel con datos reales y recursos locales.

**Non-Goals:** migrar dominio, guardar borradores o pestañas entre reinicios, rediseñar operaciones Rust, activar embeddings/LLM, copiar datos de páginas personales, clima real o control de pestañas del navegador externo.

## Decisions

1. **Catálogo de rutas separado de enlaces visibles.** Un módulo define metadatos de todos los destinos y sólo seis entradas primarias. Así las rutas secundarias no pierden nombre/icono al ocultarlas. Collections es un botón de cuadrícula junto al buscador. Las otras vistas se encuentran como destinos del buscador y la búsqueda de adjuntos conserva un acceso avanzado.
2. **Shell global y panel por pestaña.** App administra workspace, tema, pestañas, búsqueda y cierre global. Cada panel conserva la lógica existente de editor/colecciones en su propia instancia Svelte, montada al abrir y oculta/inert al desactivarse. Esto conserva el editor y su historial sin serializar/deserializar documentos. Se considera desmontar/rehidratar; se descarta porque arriesga buffers inválidos, revisiones y undo del editor. Lecturas y refresh de activación quedan limitados a la pestaña activa.
3. **Guardias locales con resultado y agregación.** Cada panel expone navegación/apertura/confirmación de salida y reporta título, dirty y busy. Alternar pestañas no invoca guardia porque no descarta el buffer. Cierre de pestaña/workspace/host recorre las guardias necesarias; una única autoridad controla `pending_edits`, escucha cierre nativo y previene unload. Un conflicto mantiene el diálogo y buffer. No se vacía el workspace antes de resolver todas las pestañas.
4. **Búsqueda contextual de lectura.** `search.query`, `entity.list` y `entity.get` son las únicas lecturas del diálogo. Se reutilizan filtros de tipo, archivo y contexto; no se simulan filtros Created by o respuestas IA. Las rutas coincidentes se presentan como accesos a vistas; resultados con consulta vacía provienen de entidades reales o accesos estáticos identificados. Debounce y generaciones descartan respuestas viejas; la vista previa se limpia al cambiar de selección. El resultado puede reemplazar el destino actual o abrir una pestaña; `+` usa siempre la segunda opción.
5. **Teclado y foco.** Buscador con `Cmd/Ctrl+K`; `Cmd/Ctrl+T` equivale a `+`, `Cmd/Ctrl+W` cierra la pestaña y Ctrl+Tab/Ctrl+Shift+Tab alternan. Flechas/Enter recorren búsqueda; Escape sólo cierra el diálogo superior. Tab strip con roles, nombres, estado seleccionado y controles de cierre separados. Las teclas del editor sólo actúan en el panel activo.
6. **Recursos y portada.** Se empaqueta la imagen autorizada sin URLs firmadas. Foto y ASCII se alternan mediante control discreto; preferencia local sólo de presentación. Los videos son referencias de movimiento y no se incorporan como fondos porque contienen una UI ajena y pesan más de 12 MB cada uno. El ASCII se genera en canvas desde un modelo geométrico ligero: cuerpo fusiforme, aletas, cola oscilante, proyección/perspectiva, profundidad y sombreado por caracteres/celdas. Hay límite de resolución y framerate y observación de visibilidad/tema/movimiento reducido.
7. **Movimiento de interfaz.** Tokens comunes de 120–180 ms para hover/foco y aparición de diálogo/panel activo, sin desmontar buffers al alternar. Reducir movimiento elimina desplazamientos y loops; ninguna operación espera a una animación. Por preferencia explícita del usuario la portada inicial es ASCII y la fotografía queda como alternativa.

8. **Listado principal y modo detalle.** Por corrección visual explícita del
usuario, Tasks/Notes/Projects/Tags/People presentan el listado en el área central.
Abrir un original o un borrador deja sólo el editor. El detalle no monta el listado, ni siquiera durante su carga. Sólo navegar a
una sección vuelve a mostrar su listado central. Collections usa todo el área disponible y Advanced search es una vista
central. No se eliminan listados, filtros ni creación.

9. **Detalle de entidad y proyecciones sin migración.** El icono va encima del
   título; propiedades compactas y relaciones preceden el contenido. Metadatos
   técnicos y herramientas auxiliares permanecen montados en desplegables.
   El prefijo exacto `Propiedades de Notion` + lista generado por el importador se
   separa únicamente para edición/presentación; el callback recompone el prefijo
   original e IDs al guardar. Párrafos duplicados inmediatos sólo se agrupan si
   cada línea coincide íntegramente con propiedades ya reconocidas. La ambigüedad
   deja el documento sin proyección. Un NodeView muestra tablas Markdown válidas
   como tabla escapada, manteniendo el bloque de código original y su editor en
   un desplegable. No se añaden tipos persistidos ni reglas de negocio en UI.

## Risks / Trade-offs

- [Más de un buffer pendiente] → autoridad de cierre global y pruebas con dos pestañas editadas, una inactiva y un conflicto real de CLI.
- [Respuesta tardía o atajo en panel oculto] → identidad por instancia/generación, `inert`, handlers condicionados a active y bloqueo mientras una operación protegida está en curso.
- [Memoria de paneles montados] → sólo se crea el panel al elegir un resultado; no se montan pestañas vacías; animación y refresh se suspenden al desactivar. Las pestañas duran una sesión y pueden cerrarse.
- [UI de referencia más rica que el producto] → sólo se muestran filtros/acciones implementados; la semejanza es visual, sin prometer IA o paridad funcional de Notion.
- [Cambio de navegación invalida selectores viejos] → actualizar recorridos que dependían de botones eliminados para usar búsqueda o Collections, manteniendo sus aserciones de dominio y guardias.
- [Pantalla/nativo no equivale a puente] → separar la evidencia headless con CLI/SQLite de cualquier aceptación del host; no atribuir IPC nativo al puente.

## Migration Plan

Sin migración SQLite/API. Respaldar los archivos de UI afectados fuera del checkout antes de editar. Añadir paneles/componentes y recursos, verificar con workspaces sintéticos y registrar el diff frente a ese baseline. Rollback sólo del diff de este cambio, preservando iconos y demás modificaciones preexistentes. La preferencia visual inválida usa un default recuperable; nunca contiene rutas, credenciales ni documentos.

La cámara de los tiburones observa desde arriba (elevación de 61,2°), conforme a la aclaración del usuario: silueta dorsal, ambas aletas pectorales, giros sobre el plano del agua y cola articulada.
