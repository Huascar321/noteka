# ADR 0020: representación OCR decimal y recuperación v13

Estado: adoptado e implementado; aceptación técnica aprobada. Sincronización/archivo se registran en la verificación.

Fecha: 2026-10-17.

## Contexto y decisión

El OCR aislado de ADR 0019 usa números f64 en JSON privado v1. Un sondeo sintético con 512 regiones encontró 135 diferencias de bits en la primera conversión serde_json; no demuestra daño de workspaces, donde aún no se guarda OCR. La integración requiere una representación estable sin alterar los codecs de datos existentes.

Se agrega transporte v2 con cadenas decimales emitidas por Swift, parseadas con `str::parse::<f64>()` y normalizadas una vez mediante Rust `to_string()`. Se preserva v1; no se habilitan features globales de serde_json. Confianza/bbox públicos y persistidos son cadenas canónicas acotadas, finitas y geométricamente válidas. No redondear a una precisión arbitraria ni retirar Eq de localizadores existentes.

Migración aditiva 013 añade manifiesto/chunks OCR con ownership de generación, sin modificar 001–012. El cuerpo canónico versionado contiene imagen, perfil/motor/OS, limitaciones y regiones; hash SHA-256 y chunks UTF-8 <=4 KiB/512, total <=1 MiB. Retención compartida <=32 MiB y snapshots completos <=64 MiB. OCR vacío es válido; cobertura describe procesamiento de la fuente, no transcripción completa.

Antes de migrar se crea backup consistente v12. DDL/versiones/integridad se publican en una transacción; fallo conserva origen y backup. Cliente v12 rechaza v13 antes de escribir. Restaurar backup con cliente antiguo en ruta nueva, nunca downgrade in-place. Restore/lectura verifican fuente, hash, forma, ownership y representación completa sin helper. Configuraciones desconocidas se conservan incompatibles sin reparación o ejecución. Se excluyen derivados OCR de base.v2 y se mantienen las nueve mutaciones batchables.

La selección FTS es explícita y proyecta texto reconocido por región con localizador inferido (ordinal, bbox/confianza, espacio y rango), sin indexar metadata ni alterar perfiles/hashes anteriores. El motor sólo corre por run explícito fuera del lock, sujeto a fuente/fence/lease y publicación atómica existentes. No OCR al abrir, migrar, restaurar, leer o buscar.

## Alternativas y consecuencias

Rechazadas: activar float_roundtrip global (cambia datos ajenos), redondear floats (pierde evidencia sin necesidad), almacenar sólo texto sin manifiesto (pierde procedencia), reutilizar localizadores PDF/bytes originales (incorrecto), migrar sin backup o borrar derivados para caber (pérdida silenciosa).

El decimal puede expandir notación científica: validar <=512 bytes por escalar y rechazar representación completa >1 MiB, sin truncar. Vision puede variar por OS y no garantiza orden/exactitud; conservar build y limitaciones. Memoria nativa observada no equivale a límite duro RSS/GPU. Disponibilidad fuera del macOS probado no se presume.

## Verificación completada

La aceptación de este ADR se verificó mediante pruebas v1/v2 bit a bit, migración real v12/rollback/old-client/restore, integridad/cuotas/fences/lectores/FTS, MCP y UI headless, regresión completa y gates 300/150 ms. Evidencia externa en el directorio señalado por `~/.local/share/noteka-verification/active-local-image-ocr-path.txt`. Ver diseño del cambio para trazabilidad; actualizar estado sólo al completar estas pruebas.
