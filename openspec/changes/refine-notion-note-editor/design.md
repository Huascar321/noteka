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
   disponibilidad; su variante header agrupa los tres accesos y Archive/Restore
   bajo un botón `…`, con icono/texto por fila, flechas/Home/End, Escape y foco
   al disparador. Elegir una herramienta abre su panel flotante acotado al viewport;
   cierre exterior/Escape conservan formularios. Archive/Restore llama al mismo
   handler protegido por la guardia del borrador y revisión esperada. Los demás
   detalles conservan la ubicación anterior.
8. Los seguimientos retiraron los controles inferiores redundantes sólo de Note.
   El diálogo compartido conserva handlers, bloqueo y foco inicial en Cancel;
   cambia el orden DOM a Cancel/Discard/Save and continue, añade 24px sobre las
   acciones y usa close/Trash2/Save del catálogo local. El botón primario mantiene
   alineación derecha si el ancho obliga a envolver la fila.
9. Los controles laterales siguen la franja horizontal del bloque, incluidos los
   márgenes de su propio panel de Note; la selección y el foco de escritura no
   los activan. Se valida la superficie visible bajo el puntero y se proyecta su
   coordenada horizontal sobre el editor para resolver el bloque. Su altura real
   excluye huecos entre bloques y el vacío inferior. Popovers, otros paneles y
   controles superpuestos no amplían el hover. Se actualiza al mover el puntero,
   desplazar/editar y perder foco de ventana; teclado y arrastre conservan acceso.
10. El fondo vacío bajo el último bloque de Note permite continuar con un párrafo.
    Un clic primario sin modificadores debe empezar y terminar en ese espacio;
    arrastrar una selección, usar controles o pulsar dentro de bloques no inserta.
    Se comprueban contenedores vacíos del propio panel y el borde inferior del
    último nodo real. Un párrafo final vacío se enfoca sin mutar el documento;
    cualquier otro final recibe un párrafo nuevo con ID e historial compartidos,
    sin heredar formato de código/lista/encabezado ni ejecutar guardado automático.
    El evento pertenece a la instancia y se retira al desmontarla.

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

Seguimiento 27/10/2026: `unify-page-actions-menu` extiende el menú de acciones superior a los otros tipos de detalle, sin cambiar su composición de contenido ni extender el bloqueo de Note.
