# Design

## Context

El componente compartido muestra la ayuda en las variantes que no usan el estilo de Note.

## Goals / Non-Goals

Retirar exclusivamente la ayuda y sus estilos sin cambiar las acciones del editor.
No se modifica la experiencia de escritura ni el formato de datos.

## Decisions

Quitar el elemento del componente compartido y sus selectores CSS sin usos restantes.
Rust y SQLite conservan sus responsabilidades; no hay migración ni acceso remoto.

## Risks / Trade-offs

La ayuda deja de ser visible; los comandos y atajos existentes permanecen disponibles.
