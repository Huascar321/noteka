# Verificación del proyector PDF independiente

Cambio `add-bounded-pdf-table-projector`,21/10/2026, aceptación técnica completa6/6.
Sólo biblioteca/ejemplo de desarrollo; sin integración o nueva operación pública.
No se atribuye al proyector verificación del archivo original o del workspace.

Evidencia nueva externa en2026-10-21-phase5-191126, señalada por
`~/.local/share/noteka-verification/active-phase5-path.txt`.

- Tests6/6 por perfil: golden independiente con orden OCR distinto de filas,
  huecos/exteriores y encabezados repetidos/hostiles, fuente exacta/roundtrip,
  confianza0, rechazos de cruces/cadenas ambiguas, canónico/extra fields/header,
  límites32 columnas/64 filas/512 celdas/regiones y geometría efectiva degenerada.
- CLI de desarrollo debug/release compila y Clippy -Dwarnings pasa; no se agrega
  código muerto al núcleo ni un helper/sidecar nuevo.
- PDF propio MIT con golden previo: source SHA256
  d809528f505eeecc3127751cf1650e6c34af50863c31e7059c2819dcf5181e57.
  CoreGraphics/Vision vigentes devuelven7 regiones por perfil. Layout exacto
  seis celdas, tres filas/dos columnas y Footer exterior. Strings12.50/003 se
  conservan; cada ordinal aparece una vez. Sin normalización/corrección OCR.
- Framing: vacío, objeto incompleto, JSON extra, campo workspace y2MiB+1 rechazados;
  cruce de columna falla completo. stdout contiene sólo razón, stderr vacío.
- Release adicional sintético:512 regiones/celdas,938150 bytes de entrada,
  5 warmups/20 muestras, p95 27,275459ms frente a300ms. Cada ejecución contrasta
  evidencia exacta y todos los ordinales; no se presenta como512 regiones nativas.

Resultados completos en table-native-*/results.json, golden-before-ocr.json,
actual-ocr.json, input.json, actual-cells.json, layout.json y benchmark-input.json.
Logs y fallos anteriores de XLSX son independientes y se conservan. Producto
SQLite16/API56/MCP28/48/49/batch9; Home9/17 no editado. Gates de producto intactos
query195,630542ms/lock17,499958ms; no cambios de caps/deadlines/validación.

Contraste final:494 paquetes previos de Cargo.lock byte-idénticos, un único paquete
nuevo noteka-pdf-table y una única incorporación al workspace. Catálogos completos
debug/release idénticos al cierre XLSX56. Home/nativos:30 fuentes protegidas exactas.
19/20 binarios iguales al pin previo a la regresión XLSX; CLIdebug reconstruido
antes de este componente tiene catálogo idéntico y repitió77 casos/108 hojas/115
páginas sin helper. No se atribuye reproducibilidad binaria al CLI. Ver
range-binary-comparison.json y table-preservation.json.

Revisión de seis escenarios/tres requisitos, diseño y diff: conservación total
por referencias, orden determinista, cruces/ambigüedad cerrados, source/bytes
validados y límites previos intactos. Ejemplo no es herramienta del producto ni
certifica archivo/workspace. No llamadas de red, escritura, rutas o modelos.
Seis tests por perfil, corpus real por perfil, recursos yClippy/fmt aprobados.
Sync/archivo únicamente del delta verificado tras validación estricta/contraste.
Integración fuente/generación/operación/UI/FTS será otro cambio;5a no completa.


Cierre: único delta local-extraction staged/validado y byte-idéntico a main,
tres requisitos/seis escenarios nuevos, contenido heredado íntegro. Archivado
como2026-10-21-add-bounded-pdf-table-projector6/6. [ADR0026](decisions/0026-bounded-pdf-table-projector.md).
Home sigue activo9/17. La propuesta de integración siguiente no es comportamiento
implementado y no se sincroniza anticipadamente.
