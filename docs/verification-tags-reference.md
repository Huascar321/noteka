# Revisión acotada de Tags — 23/10/2026

## Alcance

`match-home-tags-reference`: sección/filas de Tags, Entity, metadatos opcionales,
vistas y resúmenes Rust. Se conservó el checkout previo. Fuentes, baseline,
backups e informes privados: `~/.local/share/noteka-verification/2026-10-23-tags-reference`.

Se verificaron las quince filas de Tags del CSV/manifiesto y sus propiedades
originales; seis páginas auxiliares/template no se confundieron con tags tipados.
Quince procedencias, dos favoritos, dos tags archivados en origen; trece visibles.
189 operaciones de relación dieron 181 vínculos nuevos y ocho idempotentes.
Cero identidades omitidas o diferencias de clasificación. Diez asignaciones de
icono observado, conservando elecciones existentes. Ningún contador se copió.

El contraste detectó dos notas largas mal clasificadas como Pages por truncar a
240 caracteres el índice CSV del helper anterior. Se corrigió a títulos completos
y se aplicó la reparación explícita de ambas identidades con un segundo backup.
Los dieciséis vínculos revisados por ese helper fueron idempotentes. Los informes
de ambas aplicaciones confirman 1.802 IDs y digest documental idénticos. No se
abrieron cuerpos de la referencia ni se escribieron páginas remotas.

## Compilación e inspección

Rust CLI compiló con `cargo build --locked --offline -p noteka-cli`; contratos
regenerados. `npm run build` final: cero errores/advertencias Svelte. Se conserva
la advertencia previa Vite por bundle mayor a 500 kB.
Bundle revisado: `index-DHVMYfMA.js`, 701,08 kB sin gzip.

Inspección manual asistida en Chrome headless, UI de producción, puente de sólo
lectura a CLI Rust y snapshot real separado. Types inicial: trece filas, grupos
Area 6/Resource 7/Entity 0 y 21 imágenes locales. Los trece contadores y las doce
últimas notas coinciden por identidad con la referencia. Valores en orden:
14, 3, 12, 1, 34, 5, 0, 20, 8, 11, 9, 21 y 8; suman 146 apariciones activas.

Vistas: Recent 13, Fav. 2, A-Z 13 y Types 13. Favoritos contrastados con la vista
real: CRE y Personal. Recent usa actividad acreditada/notas locales y no reproduce
toda la fórmula de actividad de proyectos de Notion; su orden se declara acotado.
Clips muestra 52 tras reparar las dos notas, Notes 132 y All conserva 1.172.

Claro, oscuro y 420 px inspeccionados; cero overflow global, cero errores del
navegador y cero requests externos. Se revisaron guardias/callbacks y la retención
de metadata al editar. No se guardaron entidades de prueba ni se hicieron nuevas
pruebas nativas Tauri/WebKit. No se añadieron ni ejecutaron suites no solicitadas.

Logs/capturas ignorados: `tags-cargo-final.log`, `tags-build-final.log`,
`tags-recovery-apply.log`, `tags-note-repair-apply.log`,
`tags-preview-accepted.log` y `tags-reference-{light,dark,narrow}.png`.
No se acredita sincronización continua, jerarquías completas o todas las fórmulas
de Notion. La revisión visual no sustituye una suite de regresión.

## OpenSpec

Cuatro tareas; tres requisitos desktop-shell y uno knowledge-entities. Se
sincroniza sólo lo implementado después del contraste de código/requisitos.

Cierre: 4/4 tareas, tres requisitos desktop-shell y uno knowledge-entities
sincronizados/contrastados. Los 31/10 requisitos previos se conservaron completos
y las otras quince specs mantuvieron sus hashes. Archivo
`2026-10-23-match-home-tags-reference`.

La validación estricta final detectó un requisito de Tags demasiado largo. Se
compactó el texto y se movió la explicación de actividad/snapshot a un escenario
en main y delta archivado, sin cambiar código ni alcance. El primer fallo se
conserva en `icons-final-specs-validation.log`.
