# ADR 0016: lector XLSX acotado antes de integrar persistencia

- Fecha: 15/10/2026.
- Estado: adoptado para el lector independiente; integración pendiente.

## Contexto

Excel local está previsto en 5a, pero el núcleo vigente sólo tiene extracción
UTF-8/PDF y localizadores FTS para esos dos tipos. Mezclar un parser nuevo con
migración/backup, una lectura pública y ampliación MCP haría difícil aislar
errores de representación de errores de publicación. No hay archivos personales
autorizados como corpus ni se necesita activar una UI para comprobar lectura.

Los valores numéricos requieren evidencia exacta. Calamine 0.36.1 tiene streaming
y lectura de fórmulas, pero `DataRef::Float(f64)` no conserva el lexema original.
Su uso directo más un `to_string` introduciría una transformación no declarada.

## Decisión

1. Entregar primero `noteka-xlsx-worker` + `noteka-xlsx-protocol` independientes,
   sin selección en la cola, API pública, FTS, UI o cambios en SQLite. El helper
   recibe bytes acotados por stdin, no una ruta ni un workspace. Su existencia
   no se presenta como soporte Excel completo o extractor ya integrado.
2. OOXML Transitional cerrado sobre zip =9.0.3 y roxmltree =0.21.1. Se valida
   estructura ZIP antes de construir el lector, CRC, consumo exacto de Deflate,
   tamaños reales, XML, namespaces, content types y todas las relaciones.
   Sin extracción a disco, DTD/resolver, macros, conexiones ni relaciones externas.
3. Números como strings léxicos, fórmula/metadatos separados de caché opcional,
   shared sin expansión, array sin cálculo. Preservar coordenadas dispersas,
   hojas/visibilidad, dimensión declarada versus observada, merges no expandidos,
   estilos numéricos declarados y sistema de fechas. Sin inferir encabezados,
   calcular estilo efectivo, renderizar o convertir seriales a calendario.
4. Restricciones de representación y recursos explícitas, con rechazo completo
   en vez de truncamiento/fallback. Se extrae la contención existente de PDF a
   `noteka-parser-runtime`, sin cambiar allocator/rlimits/watchdog ni protocolo
   PDF. El heap solicitado no se confunde con RSS ni con sandbox de seguridad.
5. Corpus Python propio con golden semántico independiente, hashes fijados,
   pruebas por parser/helper/validador reales y regresión PDF debug/release.
   Medición de proceso real separada de la aceptación funcional.

## Alternativas descartadas para este recorte

- Integrar primero y validar después: arriesga persistir una representación con
  pérdida o anunciar un extractor sin evidencia.
- Calamine y round-trip por f64: no cumple la preservación léxica elegida. No es
  una crítica general a calamine ni un bloqueo para otros usos futuros.
- XML propio: innecesario y difícil de auditar. quick-xml es alternativa legítima
  pero requiere más estado para este árbol pequeño y acotado.
- Descomprimir a carpetas temporales o invocar Excel/LibreOffice: amplía archivos,
  superficie ejecutable y contención sin aportar una necesidad del subconjunto.
- Renunciar a límites para aceptar libros comunes grandes: no se promete Excel
  genérico; una ampliación deberá preservar evidencia y medirse por separado.

## Consecuencias y continuación

Un XLSX válido puede quedar fuera del subconjunto. Los motivos y limitaciones
deben quedar visibles; caché no significa valor recalculado y una hoja sin celdas
no certifica documento visual vacío. El original sigue siendo la autoridad.

El siguiente cambio debe diseñar migración aditiva, formato persistido, integridad,
retención compartida, lector público con localizadores y supervisor con fences/
cancelación/kill/wait antes de habilitar XLSX en la cola. No se reutilizarán offsets
UTF-8 de archivo para celdas ni localizadores PDF. Mantener contrato previo,
recuperación y presupuesto de búsqueda; no indexar XLSX por una ruta que suponga PDF.

Ver [guía](../xlsx-reader.md), [verificación](../verification-xlsx-reader.md) y
artefactos del cambio `add-bounded-xlsx-reader` para límites/evidencia exactos.
