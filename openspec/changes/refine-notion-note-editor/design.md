# Design

## Context

Ver proposal.md. BlockEditor usa Tiptap 3, adaptador semántico v1/v2/v3 y IDs
UniqueID; WorkspacePane conserva borrador, prefijo importado y CAS al guardar.
Las marcas soportadas son bold/italic/strike/code/link. No se adopta underline,
color, callouts o tipos de bloque que Rust todavía no admite.

## Goals / Non-Goals

**Goals:** herramientas junto a selección/bloque, teclado, geometría acotada,
identidades e historial conservados y estilos limitados a Notes normales.

**Non-Goals:** cambiar el dominio, persistencia, prefijos, guardias o autosave;
implementar otros módulos de Notion o la propuesta Meeting Notes.

## Decisions

1. Activación explícita de apariencia de página en la invocación de Note normal.
   El editor compartido conserva su presentación anterior para Task/Goal/Milestone.
2. Menús locales sobre el Tiptap actual; no se añade otra librería/editor. Se
   calculan posiciones desde coordenadas de selección/DOM, con límites de viewport,
   actualización al scroll/resize y cierre/foco restaurado con Escape.
3. El menú `/` reconoce un comando al inicio de un bloque o después de espacio,
   fuera de código, con selección vacía. Filtra opciones existentes y elimina sólo
   el rango del comando al aplicar. IME, barras de URL y Escape no se interceptan.
4. Acciones sobre hijos directos del documento: una lista es un bloque completo.
   Movimiento usa transacción ProseMirror; duplicación renueva todos los IDs.
   Arrastre interno mantiene el mismo nodo y tiene alternativa por botones.
5. La barra de selección usa las marcas existentes; enlace/adjunto y vista tienen
   paneles explícitos. Abrirlos no muta documentos ni importa originales. Los
   handlers preservan la selección al hacer clic en controles.
6. SQLite/Rust y el adaptador continúan siendo la autoridad. Sólo las transacciones
   de contenido notifican cambios; estado visual, selección y hover son efímeros.
7. Por ajuste solicitado durante la implementación, PageTools de Note normal se
   ubica en la barra superior. El mismo componente conserva paneles montados y
   disponibilidad; su variante header usa panel flotante acotado al viewport,
   Escape con foco al icono y cierre exterior sin borrar formularios. Los demás
   detalles conservan la ubicación anterior.
8. Los seguimientos retiraron los controles inferiores redundantes sólo de Note.
   El diálogo compartido conserva handlers, bloqueo y foco inicial en Cancel;
   cambia el orden DOM a Cancel/Discard/Save and continue, añade 24px sobre las
   acciones y usa close/Trash2/Save del catálogo local. El botón primario mantiene
   alineación derecha si el ancho obliga a envolver la fila.
9. Por pedido posterior, los controles laterales siguen el hover real del bloque,
   no la selección ni el foco de escritura. El puntero se valida contra el DOM y
   los límites del bloque; un puente de 3px permite alcanzar los controles sin
   perderlos. Se actualiza al mover puntero, desplazar/editar y perder foco de
   ventana; el foco explícito de teclado y el arrastre mantienen acceso operativo.

## Risks / Trade-offs

- Scroll/zoom/paneles montados inactivos → límites del viewport, cierre al perder
  contexto y revisión con notas largas, dos temas y ancho estrecho.
- IDs repetidos/historial por mover → conservar el nodo al mover y renovar IDs
  recursivamente al copiar; verificar guardado/reapertura con CLI/SQLite sintéticos.
- Barra que roba selección → prevenir mousedown en acciones y guardar rango para
  formularios de enlace; revisión de ratón/teclado y Escape.
- HMR y datos personales → conservar copias de fuentes actuales y usar workspace
  sintético aislado para la revisión, sin recorrer ni editar notas del usuario.

## Migration Plan

No cambia formato ni esquema. Deshacer las modificaciones de presentación revierte
la experiencia; los documentos guardados siguen siendo v1/v2/v3 compatibles.
Verificar compilación y flujo en sesión headless propia; no usar pantalla nativa.
