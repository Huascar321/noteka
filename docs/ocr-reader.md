# Lector OCR macOS independiente

`add-bounded-macos-ocr-reader` está implementado/verificado, sincronizado y archivado el 17/10 (6/6 tareas). No está
integrado en la cola, SQLite, API, MCP, UI o búsqueda. La fuente original y los
contratos v12/API v1/52 permanecen intactos. Este lector previo no cierra 5a.

El ejecutable recibe bytes con un header privado, sin rutas o workspace, y devuelve
JSON. Usa Apple Vision revision 3, perfil **fast**, idiomas **es-ES/en-US**, sin
corrección ni detección automática de idioma. No cambia a accurate ni descarga
recursos cuando falta soporte. Requiere macOS 13 o posterior con el perfil
disponible; la aceptación real se realiza sobre el OS documentado, sin afirmar
pruebas en todas esas versiones. [ADR 0019](decisions/0019-bounded-native-ocr-reader.md).

La API oficial reconoce texto de imágenes y devuelve observaciones con candidatos;
no garantiza transcripción perfecta. [Apple Vision](https://developer.apple.com/documentation/vision/vnrecognizetextrequest),
[observaciones](https://developer.apple.com/documentation/vision/vnrecognizedtextobservation).
El perfil del producto, admisión y cuotas son decisiones de Noteka verificadas
localmente, no garantías de Apple.

## Entrada y salida

Header de 24 bytes: `NKOCR001`, versión u16 little-endian 1, dos bytes reservados
cero, source_bytes u32, remaining_ms u32 y cuatro bytes reservados cero; siguen
los bytes exactos y EOF. Plazo 1–5.000 ms, sólo acortable desde inicio del helper.
Header inválido o bytes adicionales produce `ocr_worker_protocol`.

PNG/JPEG de una sola imagen, sin reducir, recortar o convertir. Se inspeccionan
tipo, estado, dimensiones y orientación antes de decodificar píxeles. Se reconoce
la imagen con su orientación EXIF 1–8. La respuesta conserva SHA-256 de **todos**
los bytes fuente, dimensiones crudas/orientadas, perfil, OS/build y limitaciones.

Cada región tiene ordinal del motor, texto exacto reconocido, confianza cruda y
bbox x/y/width/height normalizados en el **marco orientado**, origen inferior
izquierdo. Son números f64 finitos, no offsets fuente o certeza semántica.
El orden de observaciones no promete orden visual/lectura. Confianza no se trata
como probabilidad de corrección. Un resultado sin regiones no demuestra ausencia
de texto, contenido visual o información. No se ejecuta texto/HTML/instrucciones.

Rust valida la respuesta completa, incluidos hash esperado, perfil, metadatos,
geometría, ordinales y cuotas. Campos desconocidos, identidades/rangos incoherentes
o exceso se rechazan enteros. No se usa el exit code como única prueba de validez.

| Recurso | Límite |
| --- | --- |
| Fuente | 1–1.048.576 bytes |
| Dimensiones | 1–4.096 por eje, área <=4.000.000 píxeles |
| Regiones | 0–512 |
| Texto por región | 1–131.072 bytes UTF-8, sin NUL |
| Representación JSON de reconocimiento | <=1 MiB, sin truncamiento |
| stdout / stderr del consumidor | 8 MiB / 8 KiB |
| CPU / watchdog | 4 s / 5 s desde inicio del helper; presupuesto sólo decrece |
| RSS máximo observado | Termina al observar >128 MiB, muestreo de 5 ms |

La vigilancia RSS **no es** un allocator instrumentado, límite duro de RSS o
contabilidad de memoria GPU. Puede observar un exceso después de producirse.
Las allocations de frameworks no pasan por el allocator Rust; las cuotas exactas
previas de PDF/XLSX y las de persistencia/índice no se modifican. Un tamaño admitido
no garantiza terminar dentro de recursos en toda máquina/carga.

Los errores de protocolo/reconocimiento son enums fijos. Se reserva un descriptor
para el protocolo y se descartan stdout/stderr de frameworks para evitar que
diagnósticos nativos reflejen contenido. Exit 70 indica setup, 73 exceso RSS
observado, 74 fallo interno/canal y 75 watchdog; CPU también puede terminar por
SIGXCPU/SIGKILL. El consumidor mata/recolecta ante pipes excesivos o deadline.
El binario normal no contiene los modos de instrumentación de pruebas.

## Desarrollo sin instalaciones

Usa Rust y SDK/Swift ya instalados; el preparador no descarga toolchains, frameworks
ni pesos. Las fuentes propias son MIT; no se redistribuyen frameworks/modelos de
Apple con esa licencia. La caché de módulos debe estar fuera del checkout.

```sh
export NOTEKA_TEST_TMP="$HOME/.local/share/noteka-verification/ocr-runtime"
mkdir -p "$NOTEKA_TEST_TMP"
cargo build --offline --locked -p noteka-ocr-protocol --example validate_ocr_reply
node scripts/prepare-ocr-worker.mjs
node scripts/prepare-ocr-worker.mjs --probes
```

`--release` elige target/release; `--probes` genera otro ejecutable, exclusivamente
de prueba. No modifica hooks, bundle o configuración Tauri. La compilación usa el
nombre `swiftc` del SDK, validando su destino sin invocar swift-frontend como
driver. `-disable-sandbox` evita un sandbox anidado del plugin del compilador bajo
el deny-network externo de verificación; no modifica la contención del ejecutable
ni permite red al compilar bajo esa política.

`tests/fixtures/ocr/generate.swift` genera imágenes sintéticas/manifest fuera de
Git, sin ventanas. Los textos esperados se declaran antes de ejecutar OCR y no se
ajustan al resultado. El runner exige `--corpus` externo existente y `--output`
nuevo dentro de NOTEKA_TEST_TMP. Ejemplo sobre un corpus ya generado:

```sh
python3 -B scripts/test-ocr-reader.py --profile debug \
  --corpus "$NOTEKA_TEST_TMP/corpus" --output "$NOTEKA_TEST_TMP/run-debug"
```

[Verificación y fallos preservados](verification-ocr-reader.md). No es soporte de
PDF escaneado, tablas, manuscritos, OCR universal o inferencia remota. La siguiente
integración debe diseñar persistencia/migración/recuperación, operaciones compartidas
y selección explícita de búsqueda; no se finge que ya existen.

## Integración posterior verificada

`add-local-image-ocr` agrega transporte privado v2 decimal conservando v1 y su
corpus. El preparador ahora admite `--tauri` con target/perfil macOS, sin probes
en distribución; usa caché externa temporal si no se define NOTEKA_TEST_TMP.
Ver [formato y operaciones de la integración](image-ocr.md), con aceptación técnica aprobada.
No atribuir esas operaciones al cierre histórico del lector aislado. Sin un
encabezado completo no existe versión negociada: el helper emite su error v1 por
defecto y el consumidor v2 lo rechaza como protocolo, sin aceptar éxitos v1.
