# ADR 0019: OCR nativo descartable con contrato Rust

Fecha: 17/10/2026. Estado: adoptada para el lector independiente; integración persistente pendiente.

## Contexto

5a aún requiere OCR. Se prohíbe instalar modelos, activar proveedores o usar
pantalla durante esta continuación. Vision revision 3 está disponible en el Mac
actual; fast reconoció un probe sintético sin red, mientras accurate agotó sus
plazos. No hay Tesseract verificado en las rutas habituales; no se instala.
La aceptación de UI anterior conserva v12/API v1/52 y Home activo 9/17.

## Decisión

- Verificar un lector independiente PNG/JPEG con Vision/ImageIO/CryptoKit del OS;
Swift sólo contiene el adaptador nativo. Rust conserva validación/contratos y la
futura integración de negocio. No se enlaza Vision al host/core.
- Perfil fijo revision 3/fast/es-ES/en-US sin corrección, detección de idioma,
fallback o descarga. La indisponibilidad es un error, no autorización para otro
proveedor. Los resultados son inferidos, con confianza y regiones falibles.
- Bytes y protocolo privado cerrado, sin rutas/workspaces. Hash completo de fuente,
imagen/orientación, perfil/OS/build y regiones normalizadas del marco orientado.
Rechazar campos/identidades/cuotas incoherentes; no truncar o corregir texto.
- Fuente 1 MiB, 4.096 por eje/cuatro millones de píxeles, 512 regiones, 131.072
bytes por texto y representación 1 MiB. No cambiar cuotas previas ni usar OCR como
si fuera texto literal UTF-8 del original.
- CPU 4 s/core dumps off y watchdog 5 s independiente de pipes; presupuesto sólo
decrece. Muestrear RSS máximo cada 5 ms y terminar al observar >128 MiB. Es un
método distinto del allocator instrumentado Rust y no es un límite duro de RSS/GPU;
explicitar esta limitación. PDF/XLSX y sus caps exactos no se sustituyen ni relajan.
- Canal dedicado para JSON y descarte de diagnósticos de frameworks; modos de
instrumentación sólo en otro binario compilado de pruebas, nunca en el normal.
- Calidad e integridad se verifican separadamente. Corpus sintético MIT con
esperado previo, ocho orientaciones EXIF y límite CER agregado <=5% sobre el
subconjunto limpio declarado, sin promesa sobre fotos complejas o manuscritos.

## Alternativas y consecuencias

Tesseract/pesos nuevos contradicen la restricción actual. Accurate no pasó los
probes con presupuesto; no se usa automáticamente. Incluir Vision en el proceso
de UI dificulta contención y puede bloquearlo. Tratar OCR como UTF-8/PDF textual
confundiría inferencia/localizadores con evidencia literal.

Se asume un backend inicial específico de macOS, con fallos de disponibilidad y
resultados que pueden variar por OS. No se redistribuyen pesos o frameworks Apple;
MIT cubre el código/corpus propios. La futura integración necesita diseñar
persistencia, recuperación y operaciones antes de cambiar formatos. Este cambio
no incorpora OCR a jobs/FTS/UI ni completa 5a, 5b o las fases siguientes.
