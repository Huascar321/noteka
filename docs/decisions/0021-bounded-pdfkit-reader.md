# ADR 0021: lector PDFKit independiente y acotado

Estado: adoptado tras la aceptación técnica del cambio
add-bounded-pdfkit-reader. Fecha local: 17 de octubre de 2026.

## Context

El lector PDF estricto previo conserva su subconjunto. Un PDF propio con Unicode
es rechazado por ese lector y reconocido por PDFKit; la causa del rechazo previo
no se demostró. Ampliar cobertura mediante otro backend requiere identificar su
fidelidad y presupuesto antes de tocar almacenamiento o exponer operaciones.

## Decision

Perfil privado apple_pdfkit_text_v1: PDFKit.PDFPage.attributedString, conversión
CFString UTF16→UTF8 estricta, número físico de página, OS/build y limitaciones
fijas. No se adopta como parser de integridad, orden visual, OCR, tablas o geometría.
Se rechaza cifrado reconocido, incluso desbloqueado automáticamente por el OS.

Helper Swift descartable separado, stdin de bytes, protocolo NKPDK001 y consumidor
Rust estricto. Fuente/texto 1 MiB, 100 páginas, pipes 8 MiB/8 KiB, CPU 4 s, vida
5 s y RSS máximo observado 128 MiB a intervalos de 5 ms y antes de emitir. El
muestreo no es límite duro RSS/GPU. Los diagnósticos nativos no pasan al cliente.

Preparación explícita con SDK instalado por perfil/arquitectura, sin descargas,
staging Tauri o integración con el núcleo. Corpus propio con expectativas humanas
anteriores al lector. Probes se compilan en otro binario y el normal rechaza flags.

## Consequences

El backend del sistema puede variar entre OS y recuperar anomalías internas; el
framing cabecera/EOF no certifica todos los objetos/xrefs. Se conserva procedencia
y no hay fallback automático entre perfiles. Un resultado no_text no demuestra
ausencia visual. La integración requiere otro cambio con recuperación antes de
DDL. SQLite 13, API 53, MCP 25/45/46, originales, cuotas y Home no cambian aquí.

Alternativas: ampliar únicamente el parser previo deja el subconjunto Unicode
sin resolver; sustituirlo de forma automática borraría diferencias de fidelidad.
Un parser PDF adicional no se incorpora sin evaluación independiente. No se
descargan modelos ni se activa OCR en este perfil.

## Evidence

Aceptación aprobada:326 Rust por perfil,59 casos nativos únicos por perfil y cuatro
gates consecutivos. Dos gates mixtos fallidos preservados, causa no demostrada.
Ver docs/pdfkit-reader.md y evidencia externa indicada
por ~/.local/share/noteka-verification/active-pdfkit-reader-path.txt. No atribuir
los checks previos de la aplicación a este lector o a toda la fase 5a.
