# Consulta conjunta local por grupos

`add-grouped-local-search` está implementado, integrado en App y técnicamente verificado.
El controlador usa search.query para entidades activas y extraction.search para
generaciones seleccionadas, con la misma consulta literal y veinte hits por grupo.
Los ámbitos individuales conservan filtros; no se suman scores, se mezclan cursores,
se infiere propiedad de adjuntos ni se declara un snapshot/total global.

Cada grupo tiene carga, error, página/siguiente/inicio/reintento propios. Un fallo
deja visible el otro snapshot y sólo retira su página/cursor; no es ausencia
universal ni causa reparación. Editar retira ambos grupos y no consulta.
La UI nunca corta texto para superar validación Rust ni prepara/indexa fuentes.

Notas conserva ID, revisión, bloque y excerpt, y abrir delega en las guardias App.
Adjuntos conserva texto canónico, intento anterior, procedencia/localizador y
cobertura completos. Es texto escapado, no HTML, fórmula ejecutable ni consulta a
proveedores. No hay filtros de entidad/archivo/contexto aplicados a adjuntos.

El cupo de adjuntos se reutiliza del lector existente. El lector conjunto de notas
tiene un cupo físico compartido entre montajes; descartar no libera IPC iniciado.
Las esperas canceladas no ejecutan. La consulta no guarda ni reemplaza buffers.

Prueba de cliente:

```sh
node --experimental-strip-types --test tests/grouped-text-search.mjs
```

Ese comando prueba el cliente. La aceptación UI se registra por separado abajo;
ninguna de las dos pruebas acredita recuperación semántica/5b o IPC nativo.

Aceptación UI integrada: **238 checks** (119 por tema) y **9 tests de cliente**
aprobados. Compara tres páginas de entidades y dos de adjuntos con CLI, texto y
procedencia/localizador/cobertura exactos, error parcial, cursor de índice vencido,
reintento independiente, query excesiva, respuesta tardía y buffer/guardia.
Fixture sintética previa intacta más 23 notas nuevas por operaciones públicas;
ningún write de UI ni preparación/indexación automática. Cero requests externos.
Evidencia en `active-unified-search-path.txt`, `logs/headless-final.log` y
`runtime/noteka-headless-gOuLXR/`. No es recuperación semántica ni aceptación nativa.
La [aceptación conjunta](verification-local-preparation-closure.md) registra regresión/gates y revisión; su delta está sincronizado/contrastado y el cambio archivado el 17/10.
