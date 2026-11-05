# Design

## Context

Inbox manual es una decisión previa confirmada por el usuario. Notion también filtra por Content, no modelado localmente. La nueva vista no presenta esa ausencia como un hecho.

## Goals / Non-Goals

Clasificación automática según relaciones locales existentes, incluso con destino archivado. No borrar ni crear relaciones, modificar Inbox o afirmar paridad completa.

## Decisions

Las consultas `tasks_unclassified` y `notes_unclassified` filtran antes de paginar dentro del snapshot y límites de Home. Task exige estado pendiente y ningún vínculo Project; Note exige pertenencia a biblioteca sin vínculo Tag/Project/People. Ante relación incompatible se devuelve error y no se presupone ausencia. La UI identifica la limitación de Content en la vista.

## Risks / Trade-offs

El resultado es una aproximación local: un vínculo Content remoto no se conoce. Las consultas y contratos son aditivos; clientes antiguos necesitan regenerar el enum para solicitarlas.
