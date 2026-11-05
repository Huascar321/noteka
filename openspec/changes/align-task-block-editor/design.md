# Design

## Context

Task y Note usan BlockEditor y el mismo adaptador, pero sólo Note activa pageStyle. El gesto inferior identifica contenedores específicos de Note.

## Goals / Non-Goals

Compartir interacción y presentación de bloques en Task y Note. Conservar propiedades, estructuras iniciales, borrador y guardado de cada entidad. Goal y Milestone quedan fuera del alcance.

## Decisions

Activar pageStyle también para Task y reconocer sus contenedores en el gesto inferior, con las mismas guardias y aislamiento por panel. Compartir estilos del editor sin alterar cabecera ni propiedades de Task. Se conservan Rust, SQLite y documentos v1/v2/v3; no hay migración ni uso remoto.

## Risks / Trade-offs

Validar gestos, foco y geometría en paneles y contenido vacío; comprobar persistencia y guardias con SQLite sintético. No usar datos personales para pruebas.
