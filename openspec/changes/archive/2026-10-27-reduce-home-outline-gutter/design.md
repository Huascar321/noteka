# Design

## Context

Home reserva 68 px dentro del scroller para un rail de 24 px, su margen y el scrollbar. El panel lateral tiene borde ajustable independiente.

## Goals / Non-Goals

**Goals:** recuperar ancho utilizable en Home sin tapar controles ni alterar el indicador.

**Non-Goals:** modificar tamaño o disposición de Side peek, indicadores de Note o comportamiento de scroll.

## Decisions

Reducir a 36 px el padding final de `.home-content` cuando Home tiene indicador. El rail conserva 24 px y su desplazamiento propio; el scrollbar queda fuera del área de contenido. Con scrollbar clásico u overlay queda separación para el indicador. Cuando el panel de Home mide hasta 300 px, adaptar por ancho del contenedor los títulos y la barra de acciones para evitar que choquen con el rail. El cambio es CSS, sin migración ni datos.

## Risks / Trade-offs

- [Elementos al borde o scrollbar distinto] → comprobar geometría, clics y viewport estrecho en ambos temas y motores; aceptación nativa queda pendiente.
