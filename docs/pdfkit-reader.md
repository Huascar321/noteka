# Lector PDF nativo independiente

Cambio activo add-bounded-pdfkit-reader. Implementado y verificado; sincronizado/contrastado y archivado como2026-10-17-add-bounded-pdfkit-reader, sin integración
con el núcleo. No está disponible en jobs, CLI/MCP, búsqueda o interfaz. Baseline
preservado: SQLite 13, API v1/53, MCP 25/45/46 y nueve mutaciones batchables.
Home sigue 9/17. Esto no cierra la fase 5a.

El perfil apple_pdfkit_text_v1 utiliza PDFKit del SDK instalado, sin descargas ni
modelos. Fuente y texto total hasta 1 MiB, 1–100 páginas. Texto de selección del
API, sin normalización propia, OCR, tablas o geometría. Cifrado reconocido se
rechaza incluso desbloqueado. El framing no valida toda la integridad del PDF;
el parser del sistema puede tolerar anomalías internas y variar entre OS.

Protocolo privado NKPDK001, cabecera de 24 bytes little-endian: magic[8], versión
u16=1, reservado u16=0, source_bytes u32, remaining_ms u32 y reservado u32=0.
Fuente 1..1048576, plazo 1..5000 ms. JSON estricto sin campos desconocidos; hash
SHA256 esperado, perfil/versiones, procedencia real y páginas consecutivas con
estado text/no_text y conteo exacto. stdout hasta 8 MiB, stderr hasta 8 KiB.

Contención implementada: core dumps desactivados, CPU 4 s, watchdog 5 s desde inicio,
RSS máximo observado 128 MiB muestreado cada 5 ms y antes de publicar. No es límite
duro RSS/GPU ni allocator Rust. El consumidor cierra/recolecta sólo su proceso.
Los diagnósticos nativos se descartan preservando un canal exclusivo del protocolo.

El consumidor Rust verifica todas las páginas, estados, límites, metadatos y hash;
el runner de aceptación limita ambos pipes, usa CWD temporal/entorno mínimo y
termina/recolecta únicamente su proceso/grupo. No hay consumidor del producto en
este recorte. Los errores del helper son razones tipadas sin texto de la fuente.

## Reproducción

Con el toolchain local ya preparado según development.md, definir NOTEKA_TEST_TMP
como un directorio externo existente. No reutilizar una carpeta de resultados.

```sh
node scripts/prepare-pdfkit-worker.mjs
node scripts/prepare-pdfkit-worker.mjs --probes
node scripts/prepare-pdfkit-worker.mjs --release
node scripts/prepare-pdfkit-worker.mjs --release --probes
cargo test --offline -p noteka-pdfkit-protocol --all-targets
cargo test --offline --release -p noteka-pdfkit-protocol --all-targets
cargo build --offline -p noteka-pdfkit-protocol --example validate_pdfkit_reply
cargo build --offline --release -p noteka-pdfkit-protocol --example validate_pdfkit_reply
xcrun swiftc -swift-version 6 -disable-sandbox \
  -module-cache-path "$NOTEKA_TEST_TMP/swift-module-cache" \
  tests/fixtures/pdfkit/generate.swift -o "$NOTEKA_TEST_TMP/generate-pdfkit"
"$NOTEKA_TEST_TMP/generate-pdfkit" "$NOTEKA_TEST_TMP/corpus"
python3 scripts/test-pdfkit-reader.py --profile debug \
  --corpus "$NOTEKA_TEST_TMP/corpus" --output "$NOTEKA_TEST_TMP/debug-check"
python3 scripts/test-pdfkit-reader.py --profile release \
  --corpus "$NOTEKA_TEST_TMP/corpus" --output "$NOTEKA_TEST_TMP/release-check"
```

El preparador acepta sólo --release/--probes y CARGO_BUILD_TARGET aarch64 o x86_64
apple-darwin; no acepta --tauri ni hace staging. La aceptación ejecuta los hijos
bajo sandbox-exec con red denegada. El -disable-sandbox corresponde únicamente al
plugin del compilador Swift dentro del sandbox externo; no desactiva las guardias
del ejecutable. La preparación verificada también se ejecutó con red denegada.

El corpus propio declara expectativas humanas antes de ejecutar el helper;
ninguna salida del lector se usa como golden. Evidencia externa:
active-pdfkit-reader-path.txt bajo ~/.local/share/noteka-verification. El límite
horario vigente se conserva allí. Ver ADR0021 y verification-pdfkit-reader.md.

Referencias del API: [PDFPage.attributedString](https://developer.apple.com/documentation/pdfkit/pdfpage/attributedstring)
y [PDFDocument.isLocked](https://developer.apple.com/documentation/pdfkit/pdfdocument/islocked).
Apple documenta el intento interno de contraseña vacía; el helper no llama unlock
y rechaza también isEncrypted aunque isLocked sea falso.
