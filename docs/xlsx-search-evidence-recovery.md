# Recuperación de evidencia de búsqueda XLSX

Fecha: **16/10/2026**. Incidente detectado con `add-local-xlsx-search` activo,
15/19 tareas históricamente verificadas y sus cinco deltas aún sin sincronizar.
**Estado final: 19/19 tareas, cinco deltas sincronizados/contrastados y
[cambio archivado](../openspec/changes/archive/2026-10-16-add-local-xlsx-search/proposal.md).**
El usuario confirmó que no tiene copia y **autorizó la revalidación alternativa**
descripta abajo. No se interpreta como autorización de publicación, modelos,
proveedores, credenciales o aceptación nativa de Home.

**Resultado:** revalidación fresca aprobada, con evidencia durable bajo
`~/.local/share/noteka-verification/xlsx-search-20261016.fikBaL/`. 287 Rust por
perfil, corpus/procesos/SDK/raw/contratos actuales/host y 606 checks headless;
dos gates mixtos p95 167,02–270,74 ms y dos intactos 114,31–152,19 ms; sonda
física diez casos/22.298.624 B. [Detalle, hashes y fallos nuevos](verification-xlsx-search.md).
Esto satisface el criterio alternativo autorizado; **no recupera** el baseline,
catálogo, clientes ni logs históricos perdidos. Documentación/revisión y cierre completados.

## Hechos comprobados al reiniciar (antes de recuperar herramientas)

- El directorio de auditoría
  `/private/var/folders/8r/02kb8hk545j51f9p2zbxz96h0000gn/T/opencode/`
  existe pero está vacío. No se atribuye una causa adicional a esta observación.
- Ya no está `noteka-xlsx-search-plan.diMsYO/env.sh`, ni las copias/manifiestos
  independientes de 369/379 archivos, ocho binarios v11, catálogo, fixtures,
  sonda física o logs completos que residían allí. Los baselines v10/v9 y la
  toolchain temporal referenciados también estaban bajo ese mismo directorio.
- La cadena `final5-*` fue notificada como cancelada por reinicio del servidor;
  no hay resultado final recuperado. No se supone que haya terminado ni aprobado.
- Tampoco está el archivo de salida administrado de esa ejecución en la ruta
  conocida de OpenCode. La búsqueda acotada en este checkout no encontró una
  copia de los baselines/logs. No se inspeccionaron backups personales ni otros
  repositorios.
- Git no tiene commits ni archivos registrados; `git fsck --full --no-reflogs`
  no encontró objetos recuperables. No hay un diff Git que sustituya el baseline.
- `cargo` no está en PATH ni en las ubicaciones locales habituales comprobadas;
  la instalación temporal de Rust/Cargo desapareció. Node y los binarios del
  proyecto en `target/` siguen presentes.

## Qué evidencia permanece

La documentación y el registro de conversación conservan resultados observados,
tablas y algunos hashes, **no los artefactos externos completos**. Los resultados
anteriores no se declaran falsos, pero sus enlaces a temporales ya no son
reproducibles desde esas rutas. No se pueden prometer todos los intentos crudos
retenidos ni repetir ahora la auditoría independiente.

El CLI release que permanecía al reiniciar tenía SHA-256
`1324fb037f358e4da4b7760618e3a632d22b28f3cbb59817bb42977026103f59`,
igual al registrado en la medición física anterior. Esto identifica ese binario
v12; **no** lo convierte en un baseline v11 ni acreditaba la regresión entonces pendiente.
La nueva compilación y su hash se registran en la verificación fresca.
El código, tareas, deltas y fixtures sintéticas del repositorio siguen disponibles.

Checks documentales iniciales del 16/10, **antes de la revalidación**: OpenSpec estricto **15/15** y **198 enlaces locales en
nueve documentos** aprobados. Se comprobó el checklist XLSX 15/19, el veto XLSX
aún presente en main (sin sincronización) y Home activo 9/17. Son comprobaciones
documentales; **no** se ejecutó una nueva regresión de la aplicación ni se
recuperaron con ellas los baselines o logs ausentes.

## Criterio alternativo autorizado

La autorización responde a la propuesta de repetir pruebas/benchmarks con los
mismos límites/targets, guardar evidencia nueva durable y documentar que ya no
puede demostrarse la comparación exacta con v11. **Se levanta ese bloqueo de
proceso, no se declara recuperada la evidencia.**

- Captura íntegra del estado **actual v12** antes de esta revalidación, manifiesto
  SHA-256 y allowlist de las nuevas ediciones; sin usarla como baseline histórico.
- Restaurar Rust 1.99.0 y dependencias locked. Repetir fmt/check/Clippy,
  debug/release, corpus/oracles/procesos/SDK/raw, contratos generados, host
  no-bundle y UI headless. No se modifican parsers, Home, corpus o aserciones
  para obtener resultados favorables.
- Repetir dos gates mixtos y dos UTF-8/PDF consecutivos sin cargas competidoras,
  con las mismas recetas/pins y objetivos 300/150 ms. Recursos físicos medidos
  separadamente, con método/autotest y limitaciones explícitas.
- Migración adicional sobre fixtures sintéticas v10/v11 construidas con los
  SQL versionados y filas sintéticas: admisión, conservación de tablas/DDL/hashes/
  originales, rollback, cursores, recibos y backup/restore con el cliente actual.
  **No son workspaces producidos por los binarios viejos ni prueban esos clientes.**
- Revisar los 52 contratos actuales y su consistencia entre transportes; los
  tests de schemas estrictos y bindings sí se repiten. La igualdad histórica
  exacta de los otros 50 frente al catálogo v11 perdido queda como limitación,
  no como un check nuevo aprobado. Tampoco se repite la prueba de cliente viejo.
- Conservar resultados históricos documentados con etiqueta histórica, sin
  prometer que aún existen logs crudos de fallos anteriores. Los intentos nuevos
  sí se retienen completos. El cierre indicará esas limitaciones expresamente.

Ubicación durable: `~/.local/share/noteka-verification/`; el archivo
`active-xlsx-search-path.txt` identifica la corrida. Queda fuera de Git y fuera
del directorio temporal perdido. No es un backup del contenido personal.

## Condición anterior (histórica, sustituida por autorización explícita)

1. Recuperar una copia durable de la auditoría original, o indicar otra fuente
   independiente anterior a los cambios con la que se puedan cumplir los checks
   acordados. Verificar procedencia/hashes antes de usarla. No reconstruir v11
   desde v12 y presentarlo como evidencia independiente.
2. Restaurar la toolchain fijada, sin actualizar dependencias ni instalar modelos,
   y repetir las verificaciones pendientes en un directorio de evidencia durable
   fuera de Git. Conservar intentos y resultados completos.
3. Reconciliar documentación y evidencia, completar 6.1–6.3 y sólo entonces
   sincronizar/archivar según 6.4. Si no existe una fuente independiente recuperable,
   hace falta una decisión explícita sobre un criterio alternativo de aceptación;
   no se rebaja silenciosamente el actual.

No se modificaron producto, cuotas, corpus, targets ni Home por este incidente;
las specs principales sólo se sincronizaron al aceptar la revalidación, no para
eludir el bloqueo. Home sigue activo **9/17**; el comando viejo de Tauri notificado
como cancelado no se relanza y no hay nueva autorización de pantalla o remotos.
