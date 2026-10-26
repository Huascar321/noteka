# Design

## Context

Ver proposal. OCRv2 preserva escalares canónicos como strings y regiones<=512.
PDF OCR ya conserva raster/motor/página. El parser PDFKit independiente no ofrece
geometría y permanece intacto. Producto candidato16/API56 con Excel en aceptación.

## Goals / Non-Goals

**Goals:** proyección determinista verificable de evidencia, previa a integrar tablas.
**Non-Goals:** runtime nativo nuevo, persistencia, detección de columnas, cortes de texto,
verificación de archivo/workspace o nuevas operaciones en núcleo/CLI/MCP/UI.

## Decisions

1. Crate noteka-pdf-table independiente, sin dependencia de core. Input estricto
con source_sha256 lowercase64, page_number1–20, OcrRecognitionv2 íntegro,
x_edges (2–33 strings unitarios canónicos crecientes), y_min/y_max y header_row
opcional1-based. Sólo PNG orientation1 coherente con raster de PDF OCR. Validar
recognition antes de proyección; no normalizar ni editar evidencia.
2. Usar f64 sólo para comparaciones geométricas sobre escalares ya canónicos;
conservar strings fuente exactos en output. Comprobar finitud/extremos y ancho/alto
efectivos positivos. No usar centros para resolver cruces: una región debe caber
entera en una columna/rectángulo; exterior completa va a outside. Intersección
parcial o cruce produce error explícito, sin dividir una línea reconocida.
3. Agrupar regiones interiores ordenadas por centro vertical descendente, x y
ordinal. Cada fila mantiene intervalo vertical común y envolvente; una región
que solapa envolvente sin compartir intervalo común, o coincide con varias filas,
es ambigua y rechaza. Ordenar filas por su intervalo vertical descendente.
Cells guardan sólo ordinales (orden fuente); recognition completa aparece una vez.
Se conservan regiones exteriores y encabezado como referencia de fila, sin copiar
ni asignar nombres únicos. No confianza mínima ni coerción de tipos.
4. Máximos32 columnas/64 filas/512 celdas, fuente recognition<=1MiB, JSON del ejemplo
<=2MiB y salida<=2MiB. Algoritmo O(n²) acotado por512 y sin I/O en biblioteca.
Ejemplo de desarrollo no es helper de producción ni CLI de Noteka; no promete
contención de pipes/proceso. Futuro caller revalidará original/canónico y aplicará
su deadline antes de usar este componente. No se agrega código muerto al core.

## Risks / Trade-offs

- Vision puede unir una fila entera → error de columna; nunca separación inventada.
- Celdas multilínea o filas superpuestas pueden ser ambiguas → rechazo explícito.
- Geometría usa raster normalizado inferior izquierdo → sin coordenadas PDF original,
rotación automática, spans ni garantía genérica; texto reconocido puede tener errores.
- Biblioteca acepta hash declarado → no certifica filesystem ni workspace; integración
posterior debe verificar fuente/generación completos, cuotas y contratos propios.

## Migration Plan

No DDL ni cambio de formato persistente. Un miembro/paquete de workspace nuevo,
dependencias externas existentes conservadas; producto16/API56 no lo consume.
Comparar veinte binarios/pins y55 contratos previos; no modificar fuentes/gates
para aceptar el recorte. Integración posterior será otro cambio y no habilita5b aún.
