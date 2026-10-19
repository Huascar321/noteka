# Proposal

## Why

El parser PDF actual tiene un subconjunto textual estricto. Un PDF propio generado con CoreText, con acentos y CJK, devuelve `invalid_pdf` en ese lector y texto correcto mediante PDFKit local; ampliar cobertura requiere otro perfil acotado y verificable. Este recorte prepara ese lector independiente antes de elegir su persistencia e integración.

## What Changes

- Incorporar helper macOS y protocolo Rust privados para texto derivado por página, con hash fuente, OS/build y límites explícitos. No anuncia disponibilidad desde el catálogo del núcleo.
- Verificar framing de entrada, cifrado incluso desbloqueado, conversión UTF-16→UTF-8 estricta y recursos. El parser de sistema puede tolerar anomalías internas: no es validador completo ni garantiza orden visual.
- Crear corpus propio con expectativas humanas anteriores al lector, pruebas reales offline, contención/probes y preparación por perfil con SDK existente.
- Reconciliar tres requisitos previos de aislamiento/cuotas con las integraciones ya verificadas: conservan el catálogo/formato vigentes, sin congelar el producto en v12/52 ni contar sólo dos extractores. No se modifican artefactos archivados.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `local-extraction`: lector PDF nativo independiente y límites; reconciliación de aislamiento XLSX/OCR y presupuesto agregado ya implementado.

## Impact

Nuevo paquete de protocolo y helper Swift, scripts/tests y documentación/ADR0021. Cargo sólo añade el paquete local con dependencias existentes. Baseline durable v13/API53/MCP25/45/46 y16 binarios; no cambia reglas/persistencia/operaciones del núcleo ni UI/Home9/17. Preparación de staging y QA respetan hashes/perfiles del cierre anterior.

Fuera de alcance: integrar jobs/SQLite/CLI/MCP/FTS/UI, sustituir pdf_text, OCR de PDF, tablas/geometría, embeddings/agentes, modelos, proveedores/OAuth/pagos/publicación o pantalla. El usuario autorizó continuar por cambios hasta completar dependencias; después se propondrá integración con recuperación antes de DDL. Este cambio no completa5a. Disponibilidad/calidad sólo se certifican en OS/hardware/corpus realmente probados.
