# Proposal

## Why

El núcleo v12 ya busca fragmentos UTF-8/PDF/XLSX seleccionados, pero Search sólo
expone títulos y bloques de entidades. Este recorte de 5a permite consultar esa
evidencia desde la aplicación sin introducir otro motor ni automatizar extracción.

## What Changes

- Añadir un selector explícito entre búsqueda de notas/entidades y adjuntos en
  Search; mantener filtros, apertura del bloque original y guardias de notas.
  No fusionar scores, cursores o resultados de ambos índices.
- Consultar exclusivamente `extraction.search` para adjuntos ya seleccionados,
  con filtros de extractor, trabajo y adjunto, páginas y cobertura del núcleo.
  Mostrar por qué cero coincidencias no equivale a buscar todo el workspace.
- Mostrar texto canónico escapado y procedencia tipada UTF-8/PDF/XLSX: IDs,
  generación, hashes, ordinal y límites. Separar caché/fórmula y señalar resultados
  anteriores; no inventar dueño, nombre de archivo, recálculo o cita visual.
- Acotar lecturas y resultados en pantalla, descartar respuestas obsoletas y
  mostrar fallos/cursor vencido sin mezclar páginas ni reparar automáticamente.
- Verificar por tests de promesas controladas y Playwright CLI headless sobre
  SQLite/CLI reales, conservando los 606 checks previos y Home independiente.

## Capabilities

### New Capabilities

Ninguna. La búsqueda de escritorio pertenece a la capability existente.

### Modified Capabilities

- `local-text-search`: ampliar «Búsqueda de escritorio offline» y especificar
  presentación, cobertura, paginación y aislamiento de evidencia de adjuntos.

## Impact

`src/App.svelte` y componentes/controlador nuevos de Search, estilos locales y
pruebas/headless bridge sintéticos. Se reutiliza `operate` y los tipos generados.
Sin dependencias, migraciones ni operaciones nuevas: SQLite v12/API v1/52,
MCP 24/44/45, schemas, núcleo Rust, parsers/corpus/gates y whitelist intactos.

Decisiones conservadas del usuario: núcleo offline, fuentes originales y reglas
Rust, desarrollo local autónomo, pruebas sin pantalla y consentimiento remoto
separado. Selector de ámbitos, UI de sólo lectura y límites de presentación son
decisiones técnicas propuestas para este recorte. No hay dependencia externa
bloqueante conocida; comprobar la UI no equivale a aceptación IPC/foco/cierre nativa.

## Non-goals

UI para importar/extraer/seleccionar/resetear adjuntos, lector completo o apertura
externa del archivo, nombres/MIME obtenidos descargando base64, búsqueda semántica,
ranking unificado, OCR/Excel/PDF genéricos, modelos, proveedores, OAuth, publicación
y cambios de Home. No completa toda 5a/5b ni las ocho tareas nativas de Home.
