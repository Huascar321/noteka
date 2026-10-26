# Design

## Context

App conserva una instancia WorkspacePane por pestaña. Cada pane contiene buffer, editor, revisión y guardia. Desmontarlo para alternar una presentación perdería historial y selección. Rust y SQLite siguen siendo dueños de operaciones/datos; este cambio sólo administra presentación local.

## Goals / Non-Goals

**Goals:** reutilizar la misma instancia, conservar contexto, integrar cierres y mantener foco/atajos inequívocos.

**Non-Goals:** peeks anidados, persistir sesiones, preferencias por vista, cambios de dominio, colaboración o nuevos formatos.

## Decisions

- Colección única de panes con relación opcional ownerId. Un peek es hijo de una pestaña y participa en las mismas guardias; sólo las raíces aparecen en la barra. Side/Center/Full son clases del mismo nodo con clave estable; New tab elimina ownerId y promueve esa instancia. Alternativa de dos árboles condicionales descartada porque remonta el editor.
- Un hijo por pestaña. Otra apertura reemplaza el destino del hijo tras su guardia; navegar a una sección cierra el hijo protegido. En un detalle, relaciones abren el siguiente original en ese mismo pane protegido. Una pestaña independiente puede convertirse en peek creando un origen de sección y manteniendo la instancia de detalle.
- Separar pane visible de pane con atajos activos. El origen sigue visible en Side peek y conserva su scroll/selecciones; ambos indicadores observan su propio scroller. Center/Full bloquean el fondo; el foco se contiene en Center sin interferir con diálogos nativos de guardia, Search o Settings.
- La activación de pestaña también se separa del foco: moverlo entre origen y peek no refresca ni desmonta filas. Full page cubre un origen montado/inactivo, con guardias disponibles en la capa modal. Ver ADR0050.
- Menú compartido en cabecera con selección actual y Default opening mode (etiqueta All views). localStorage validado `noteka.peek-mode.v1`; preferencia global inicialmente Side. Sin acceso remoto ni migración del workspace; escritura fallida conserva la preferencia durante la sesión y muestra aviso.
- Panel derecho inicialmente de aproximadamente mitad de la ventana. El seguimiento `resize-workspace-panels` reemplaza el ancho fijo por memoria local y un máximo de 65%, también en ventanas estrechas, más exclusión del menú por encima de 35%; ver [ADR0051](../../../docs/decisions/0051-panel-widths-and-suppressed-peeks.md). Center usa marco centrado y backdrop. En ambos, encabezado pegado, contenido scrolleable y movimientos breves/reducibles.

## Risks / Trade-offs

- [Buffers ocultos] → cierre agregado recorre raíces e hijos y activa el dueño antes de preguntar.
- [Atajos duplicados] → sólo el pane enfocado está activo; visibilidad del indicador se separa del foco.
- [Cambio de layout mueve texto] → medir título/outline con resize; preservar instancia, selección y posición de scroll sin reabrir datos.
- [Center bloquea menús/guardias] → trap de foco sólo sin diálogo superior; Escape respeta preventDefault y popovers.
- [Preferencia global menos granular que Notion] → interfaz lo identifica como All views; ámbito por vista queda fuera del pedido explícito.

## Migration Plan

Sin cambio SQLite/documentos. Revertir presentación deja los formatos intactos; el único valor nuevo es una preferencia descartable y validada del cliente.
