# Design

## Context

App distingue panel visible de panel enfocado. Home entrega actualmente el foco a la animación; su observador y reloj ya controlan portada fuera de vista, ventana oculta y movimiento reducido.

## Goals / Non-Goals

**Goals:** mantener movimiento con Side peek visible sin modificar foco, atajos ni pausas fuera de vista.

**Non-Goals:** alterar escenas, tasas de frames, comportamiento de Center/Full, persistencia o contenido.

## Decisions

App informa al panel raíz si su Side peek está visible; Home combina esa señal con su propia visibilidad y el estado activo para gobernar sólo AsciiShark. El panel enfocado sigue siendo el único activo para teclado y acciones. Los observadores y OceanClock existentes siguen administrando visibilidad física, pausa y tiempo activo.

## Risks / Trade-offs

- [Trabajo gráfico adicional mientras la portada visible comparte pantalla] → conservar pausa por intersección, ventana oculta y movimiento reducido; verificar frames y límites sin prometer rendimiento físico.

## Migration Plan

Sin migración ni preferencias nuevas; revertir el cambio no afecta documentos.
