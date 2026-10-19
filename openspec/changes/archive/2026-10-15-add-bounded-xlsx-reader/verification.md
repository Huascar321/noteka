# Verification Report

## Summary

Revisión al 15/10/2026 sobre fuente final3 y baseline externo independiente.
Recorrido headless y regresión final completos; cierre documental revisado antes
de sincronizar/archivar. No se omiten checks por ausencia de datos personales o UI.

| Dimension | Status |
| --- | --- |
| Completeness | 8/8 tareas verificadas, seis requisitos y dieciséis escenarios trazados con evidencia externa final3. |
| Correctness | Corpus final3: 216 casos × parser/helper en debug/release, 512 mutaciones por perfil, golden independiente y límites exactos/+1. |
| Coherence | Diseño y ADR coinciden con lector independiente; contención PDF conservada, sin migración ni nuevas operaciones. |

## Implementation Evidence

| Requirement | Source | Test evidence |
| --- | --- | --- |
| Lector independiente | `crates/noteka-xlsx-worker/src/runtime.rs:6`, protocolo `src/lib.rs:38–140`; core no enlaza parser/runtime | framing crudo, hash, catálogo baseline y v10/no activación por CLI real |
| Contenedor cerrado | `crates/noteka-xlsx-worker/src/container.rs:50–254`, parser `src/parser.rs:18–117,211–401` | CRC/descriptors/consumo Deflate/rutas/activos/externos/XML; fuente y expansión frontera/+1 |
| Celdas tipadas | parser `src/parser.rs:119–208,402–477,497–661,664–783` | números léxicos, strings/shared/inline/rich/escapes, estilos/fechas, dispersión/merges y referencias incoherentes |
| Fórmulas/caché | parser `src/parser.rs:478–556`; protocolo `src/lib.rs:419–470` | normal/shared/array, caché opcional, grupos inválidos y tipos/flags no admitidos |
| Presupuestos | protocolo `src/lib.rs:5–31,254–418`; preflight y checks XML/parser | límites de todos los niveles, salidas adulteradas, representación 1 MiB exacta/+1 y cap lógico tras unescaping |
| Vida acotada | `crates/noteka-parser-runtime/src/lib.rs:9–170`, runtime XLSX | probes POSIX/heap/panic/CPU/deadline, pipes bloqueados/padre muerto, doce lectores concurrentes y regresión PDF |

Los rangos son de final3, y se acompañan con la matriz escenario por escenario y
comandos/resultados completos en [verificación](../../../../docs/verification-xlsx-reader.md).

## Review Findings Resolved

1. Declaración XML con tabulador eludía la comprobación de encoding. Reproducida
   antes del fix; ahora se reconocen los cuatro whitespace XML y se verifica UTF-8.
2. Texto/CDATA/comentarios en profundidad 64 se contaban como nivel de elemento.
   Se cuenta sólo ancestros elemento; se añadieron fronteras 64/65 sin omitir texto.
3. ST_Xstring escapado se limitaba antes de decodificar. Se aplica el cap lógico
   después del unescaping; el raw sigue acotado por parte/fuente/heap. Inline,
   shared y str tienen frontera/+1, sin subir caps ni cambiar el golden anterior.
4. Diseño decía que el harness denegaba FS. Se corrigió la afirmación: deniega red,
   comprueba cwd temporal sin escrituras y no es un sandbox FS. No se retiró un
   requisito de contención ni se atribuye al binario una garantía inexistente.

## Closure Checks

- `headless-final3-retry.log`: 592 checks + catorce de teclado/intercalados, sin
  solicitudes externas. Se usó socket temporal corto; el primer intento falló
  antes del navegador por longitud de socket de macOS, sin cambiar tests.
- Pins final3 intactos y diff contrastado con baseline externo. Ningún cambio de
  core/CLI/MCP/frontend/host/migraciones o artefactos Home; sin ampliación funcional
  del producto. Sincronizar sólo los seis requisitos implementados de este delta.
- No quedan defectos de implementación conocidos ni escenarios sin cobertura
  dentro del alcance declarado. Las limitaciones siguientes no son excepciones
  añadidas para eludir un requisito. Home conserva 9/17, sin sync/archivo.
- Auditoría presync: 316 archivos previos idénticos y diez cambios previstos;
  107 pins final3 intactos y 276 enlaces relativos válidos. OpenSpec estricto
  15/15; sin críticos o advertencias pendientes de implementación dentro del alcance.

## Limitations

No se verifica Excel genérico, cálculo/estilo visual, libros personales, integración
con la cola ni seguridad universal. Se revisaron versiones/licencias/RustSec, no se
ejecutó cargo-audit. El helper no ofrece sandbox OS de FS/red; su contención es
allocator/rlimits/watchdog y ejecución controlada. Métricas son locales al corpus,
no garantía universal. No se inició host/ventana nativa, modelos ni proveedores.
