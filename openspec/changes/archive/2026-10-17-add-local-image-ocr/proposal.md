# Proposal

## Why

El lector OCR independiente cerró 6/6 con Vision local real y protocolo validado, pero no procesa adjuntos administrados ni persiste resultados. Este recorte de 5a integra imágenes PNG/JPEG en el flujo explícito existente, con recuperación y evidencia inferida accesibles desde CLI/MCP/escritorio.

## What Changes

- `image_ocr` v1 sobre originales administrados PNG/JPEG, configuración v1 fija y perfil local r3/fast/es-ES/en-US, sin instalación, fallback, red o interpretación del texto.
- Protocolo privado v2 con coordenadas/confianza como cadenas decimales; preservar v1 independiente. El sondeo numérico detectó 135 cambios de último dígito al parsear números JSON sintéticos. Rust normaliza/valida con su parser estándar, sin activar features globales de serde_json ni modificar contratos ajenos.
- SQLite v13 aditiva: representación OCR canónica versionada, hash y chunks UTF-8, con manifest y regiones; presupuesto retenido compartido de 32 MiB. Diseñar backup/migración/restore/rollback antes de editar el formato; conservar originales, versiones desconocidas y generaciones previas ante fallo.
- Enqueue/run/cancel/retry/control y selección de generación conservan CAS, lease/fencing y ejecución explícita. Helper fuera del lock; publicaciones atómicas, sin nuevas mutaciones batchables.
- `extraction.ocr_regions`, lector estricto paginado 1–64 (32 default), manifiesto/fuente/resultado anterior y revalidación completa sin helper. API v1/53 y MCP 25/45/46, con las siete exclusiones y límites previos.
- Indexación OCR explícita con texto reconocido por región y localizadores que conservan bbox/confianza y rango derivado. No usar JSON/metadata como texto ni confundirlo con bytes originales; conservar corpus, validación completa, cuotas y gates.
- UI de preparación, lector y ámbitos Search/agrupada presentan OCR inferido y sus límites; sin acciones al abrir/buscar, pérdida de buffers o cambios Home. Preparar helper nativo sólo para target macOS; disponibilidad real falla explícitamente.

## Non-goals

No PDF escaneado/texto PDF ampliado, tablas reconstruidas, ampliación Excel, embeddings, agentes, proveedores cloud, OAuth, pago o publicación. No renderer/overlay del original, editor de transcripción, corrección de OCR, confianza como probabilidad ni garantía de exactitud/lectura visual. No instalar modelos o activar servicios remotos. Este recorte no completa toda 5a y Home sigue 9/17.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `local-extraction`: extracción OCR administrada, almacenamiento/lectura/proyección explícitos, protocolo decimal estable y presentación de preparación/lectura.
- `local-workspace`: evolución recuperable v12→v13 y validación de OCR en backup/restore, sin ejecución implícita.
- `core-operations`: lector OCR compartido y catálogo v1/53 con compatibilidad delimitada de los 52 contratos previos.
- `local-mcp`: nuevo lector de regiones en la superficie autorizada, sin cambiar ejecución/escritura o exclusiones.
- `local-text-search`: evidencia/localizadores OCR inferidos en ámbitos individual y agrupado, preservando fuentes y límites.

## Impact

Núcleo Rust, protocolo/helper nativo, migración 013, CLI/MCP, DTOs y paneles existentes con pruebas de integración/contención/recuperación/headless. Baseline durable de 488 archivos/catorce binarios y catálogo completo v12 fuera de Git. El backend Mac requiere perfil ya disponible; no se presume otro OS o permiso externo. La meta sigue hasta fase 6 por dependencias; las instalaciones/activaciones y aceptación nativa de Home siguen fuera de la autorización actual.
