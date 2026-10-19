# ADR 0012: PDF textual acotado en helper descartable

Estado: **adoptada**. Fecha: 14 de octubre de 2026.

## Contexto

La cola UTF-8 ya usa selección explícita, originales inmutables y publicación
cercada. PDF necesita un parser con asignaciones y fallos que un catch_unwind
del cliente no contiene. Extraer texto no equivale a transcribir visualmente un
PDF, y los fallbacks del loader no certifican ausencia de pérdida.

## Decisión

- `lopdf =0.45.0`, MIT y sin default features, sólo en `noteka-pdf-worker`;
  core/host no enlazan el parser. `noteka-pdf-protocol` no depende de SQLite/parser.
- Un hijo por ejecución, bytes por pipes privados acotados, entorno limpio y
  ruta sibling del ejecutable. Sin PATH/CWD search, shell del frontend, daemon,
  modelos, rutas públicas de helper ni credenciales/ownership enviados al hijo.
- PDF 1.0–1.7 no cifrado, <=1 MiB/100 páginas; doce Type1 latinas estándar,
  Standard/WinAnsi y texto directo. Font/mapping/estructura excluidos fallan
  completos; no OCR, render, ToUnicode, geometría, fuentes embebidas ni tablas.
- Auditor original independiente de xref/framing y revisiones históricas; vista
  de carga sintética efectiva sin Prev que conserva offsets y no repara objetos.
  Textos se producen con algoritmo versionado y golden propio, no extract_text.
- Heap solicitado vivo Rust del hijo <=128 MiB, CPU 4 s, core dump 0 y watchdog
  propio. Padre conserva 5 s/lease 15 s, pipes no bloqueantes y kill/reap al perder
  presupuesto o fence. **No** es cap de RSS ni sandbox de privilegios.
- `009.sql` aditiva, backup previo y lector nuevo `extraction.pdf_fragments`;
  manifest con scope/limitaciones y rangos del texto derivado por página. Lector
  literal/48 contratos previos intactos. API v1, SQLite v9/49 operaciones.
- Presupuesto mixto de 32 MiB sobre texto real; snapshot completo de 64 MiB,
  huella original base.v2, receipt/plan v1 y whitelist sin ampliar. No purga.
- Hash/reconstrucción completa de PDF fuera del escritor, en snapshot previo;
  cola/ownership/configuración/forma se revalidan al reclamar y fence/fuente/
  budget al publicar. No se omite integridad para cumplir una cifra de latencia.

## Evidencia y consecuencias

[Verificación](../verification-pdf-extraction.md): corpus propio de 235 casos,
procesos reales, CLI debug/release, migración desde CLI v8 y backup sin helper.
Benchmark M1 Max/64 GiB: 99 configuraciones con cinco warmups/veinte repeticiones;
el primer lock p95 falló a 180,54 ms, y tras separar reconstrucción del escritor
el máximo fue 20,19 ms sin ampliar caps. Son mediciones locales, no garantías
para cualquier PDF ni revisión de seguridad exhaustiva de lopdf.

Muchos PDF modernos quedan fuera: ampliar fuentes/OCR requiere otro cambio con
corpus/versionado propio. Modificación privilegiada de binarios y explotación
del proceso hijo no están aisladas por permisos. Preparación target-triple y
no-bundle no acreditan firma/notarización, distribución ni IPC nativo del host.
Home conserva sus ocho tareas nativas pendientes; esta decisión no las rebaja.

## Alternativas descartadas

Parser dentro del core: abort/OOM/stack overflow no quedan contenidos por unwind.
Pdfium/render: dependencia nativa y superficie innecesarias para texto acotado.
Daemon: lifecycle permanente no requerido. Usar salida permisiva como texto fiel:
confunde recuperación de parser con cobertura. Reutilizar offsets literales para
PDF o reemplazar SQLite/originales por texto: procedencia y autoridad incorrectas.
