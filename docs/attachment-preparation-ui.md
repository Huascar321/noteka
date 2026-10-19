# Preparación explícita de adjuntos locales

Recorte `add-local-attachment-preparation-ui` de 5a, implementado y con aceptación técnica
headless aprobada. [Verificación](verification-attachment-preparation-ui.md). No cierra toda
5a ni las ocho tareas nativas pendientes de Home.

Abrí una nota y desplegá **Preparar adjuntos locales**, debajo del editor. Elegí
una referencia o ingresá el ID administrado de un original importado con el
editor/CLI. Referencias repetidas aparecen una vez; su texto se identifica como
rótulo de bloque, puede estar pendiente de guardar y no es metadata del archivo.
Un adjunto compartido no tiene un dueño inferido. No hay listado global ni
`attachment.read`/base64 para preparar. Documentos futuros conservan su JSON y
permiten ID manual sin interpretar bloques desconocidos.

1. **Consultar trabajos** llama `extraction.list` filtrado por adjunto, hasta 20.
   **Siguiente** reemplaza la página; volver a consultar comienza sin cursor.
   Vacío no demuestra que exista el original. Seleccionar un trabajo no consulta.
2. **Encolar extracción** usa ID y extractor elegido (`utf8_text`, `pdf_text` o
   `xlsx_cells`), con defaults Rust. No infiere soporte por nombre ni ejecuta.
   El recibo informa deduplicación, ID y estado.
3. **Consultar trabajo elegido** hace get job → get control → get index filtrado
   por job. Son snapshots consecutivos independientes, no uno atómico. Muestra
   soporte/configuración, revisiones, intentos, ownership/error y generación,
   incluidos resultados de intentos anteriores. Index get valida el índice
   completo pero no comprueba originales.
4. **Reanudar cola global** cambia sólo pausa con la revisión observada. Afecta
   todo el workspace, no ejecuta. **Ejecutar trabajo elegido** siempre envía un ID;
   puede recuperar intentos vencidos del workspace conforme al núcleo. Run
   pausado da idle sin aumentar intentos. La ejecución mantiene su resultado y
   motivo explícitos; no selecciona la generación para Search.
5. Consultá de nuevo y usá **Seleccionar generación para búsqueda** para un
   resultado compatible, incluso si tiene cero fragmentos. Put envía ID de job,
   generación observada y revisión del índice. Recibo muestra changed, entrada y
   revisión; no parsea ni busca. Search sigue siendo una pantalla independiente.

Cada mutación retira inspección y exige consulta explícita antes de otra acción
dependiente. Cancelar/reencolar usan revisión del job; retry conserva resultado
previo, no ejecuta. Regenerar con éxito invalida su selección anterior y no elige
la nueva. No hay reset/purga/reparación desde este panel.

No hay polling, consultas al montar/cambiar selección/activar ventana, ni pasos
automáticos. Un error conserva código/mensaje/details y retira evidencia anterior.
Un conflicto no se rebasa; una escritura con transporte fallido puede haberse
confirmado: consultar, decidir y actuar explícitamente. No se reintenta sola.

El cupo físico de preparación es uno entre montajes. Cambiar ID/nota/workspace
descarta esperas/resultados viejos y cancela una llamada todavía en cola. Salir
no cancela Rust iniciado ni libera su cupo antes de terminar. Cancelar el job es
otra operación explícita. El panel no bloquea ni guarda el editor; navegar
mantiene guardar/descartar/cancelar y las revisiones capturadas.

Los límites completos siguen en [extracción](extraction.md),
[PDF](pdf-extraction.md), [XLSX](xlsx-extraction.md) e
[índice](attachment-search.md). La UI los explica y Rust los valida; no instala
modelos, envía contenido, recalcula Excel ni transcribe imágenes. Sin cambios de
SQLite v12, API v1/52, MCP 24/44/45 o nueve mutaciones batchables.

El hook local de Tauri prepara ambos helpers mediante `extraction:prepare` en el
perfil/target solicitado; su staging sigue ignorado por Git. El script previo
`pdf:prepare` permanece disponible. Esto no publica, firma ni certifica un bundle.

```sh
npm run test:attachment-preparation
npm run extraction:prepare -- --tauri
npm run test:headless-preparation
```

Requieren toolchain del proyecto, CLI/helpers debug ya construidos y
`NOTEKA_TEST_TMP` externo. Playwright usa navegador headless sin perfil personal;
el puente ejecuta CLI real sin red en macOS. No acredita IPC/foco/cierre nativos.
