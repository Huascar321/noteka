# Design

## Context

La fuente acredita la etiqueta `Voice Note`; `NoteType` usa `voice`. La identidad y las filas ambiguas conservan las guardias existentes.

## Goals / Non-Goals

Reconocer la etiqueta exacta tras normalizar mayúsculas y espacios. No deducir tipos por título ni convertir notas existentes por una lectura.

## Decisions

El adaptador de recuperación traduce el alias antes de validar el conjunto existente; si ya hay `note_type`, no lo reemplaza. Una aplicación futura sigue exigiendo backup y revisión esperada. No se modifica el documento ni el esquema.

## Risks / Trade-offs

Notas ya recuperadas necesitan una ejecución explícita posterior del recuperador con sus fuentes y backup; no se ejecuta sobre datos reales aquí. Tipos desconocidos siguen sin clasificarse.
