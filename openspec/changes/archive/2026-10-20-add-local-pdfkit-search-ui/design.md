# Design

## Context

Baseline14/54 verificado y archivado: páginas PDFKit canónicas, lector54 y cola
existentes. El índice rechaza ese extractor; preparación/lector/Search sólo
conocen los cuatro tipos anteriores. Cliente14 durable y fixture real con cinco
generaciones/cuatro selecciones se guardan fuera de Git antes de015. Home9/17,
App/tema y lectores nativos quedan protegidos por hashes.

## Goals / Non-Goals

**Goals:** proyectar y seleccionar explícitamente texto PDFKit, leerlo/buscarlo por
paneles existentes, mantener validación completa y recuperación reproducible.
**Non-Goals:** OCR/render/tablas, ampliar Excel, modelos/embeddings/harness, Home,
fase6 o nuevas operaciones/dependencias/helpers. No inferir orden visual o dueño.

## Decisions

### Proyección por página sobre el derivado canónico

Rust carga/valida Document completo antes de proyectar; página física1-based,
fragmentos UTF-8<=4096 consecutivos sin cruzar páginas, máximo512/1MiB. Con100 páginas/1MiB no se llega a512 fragmentos, incluso contando cortes UTF-8
de hasta3 bytes de ajuste; se verifica la fragmentación máxima admitida sin
fabricar un caso válido513. El cap defensivo512 permanece. No_text
no crea fragmentos pero conserva su posición en el lector. Locator nuevo
pdfkit_page_text con proyección pdfkit_page_text_v1, página/rango/page_bytes,
perfil, engine OS/build y limitaciones exactas. No se indexan campos de metadata.
No son offsets PDF ni geometría. Alternativas rechazadas: JSON como texto,
reutilizar pdf_page_text estricto o normalizar el texto nativo.

### Integridad, hashes y límites anteriores

Canonical verifica ownership/tipo/config/hash y carga íntegra. El hash existente
content.v1 recibe el manifiesto PDFKit sólo para ese formato; las otras ramas no
cambian bytes/hashes. Get/search verifican todo el índice y source sólo en
candidatos de página/lookahead, conservando la distinción de cobertura. Sin cachés
de confianza, cuotas/perfiles nuevos globales o validación parcial. CAS, no-op,
invalidación por publicación y cursor de revisión son los mecanismos existentes.

### Barrera15 antes de nuevas selecciones

015 sólo actualiza versiones; no añade tablas ni limpia/reindexa. Registry de
snapshots incluye la migración. El cliente14 independiente rechaza15 y el actual
rechaza16; backup14/restauración antigua y rollback antes/después de versión se
comprueban con la fixture real14 capturada antes del DDL. Originales/configs,
filas/hash/cursores de selecciones anteriores quedan iguales; sólo la cobertura
reconoce PDFKit sin seleccionar. Un recibo de lote anterior requiere preview nuevo.
Se precisan escenarios heredados para nombrar el cliente que rechaza la versión,
sin eliminar sus escenarios ni fabricar comparación histórica v11.

### Paneles compartidos, sin modificar App/Home

Añadir el tipo a selects y mapas explícitos. El lector usa extraction.pdfkit_pages,
una página de20 páginas físicas, manifiesto completo y texto/status por página.
Search y agrupada muestran locator/límites completos y texto escapado. Los pools
/lifetimes/controladores existentes conservan su cupo físico y retiro de datos
anteriores. Ninguna consulta al montar, editar, volver al foco o cambiar selección.
Sin persistencia de contenido del navegador ni callbacks de guardado nuevos.

### Compatibilidad y aceptación

Se conservan54 operaciones,52 schemas no afectados, errores/batch9/MCP26/46/47.
Sólo index/search y metadata de capacidades tienen ampliaciones identificadas.
Bindings se generan desde Rust. Corpus nativo/golden, procesos de CLI/MCP y fixture
headless reales prueban rangos, estado vacío, corrupción, CAS y preservación de
buffers. Se repite regresión relevante y los cuatro gates originales300/150;
ningún fallo habilita relajar límites. Evidencia y baselines fuera de Git.

## Risks / Trade-offs

- Manifiesto/engine repetidos en locators aumentan respuesta: cap2MiB comprobado,
  sin subir límites ni truncar; payload completo no significa cobertura visual.
- Cliente14 entendería tablas pero no la proyección: barrera15 evita leer postings
  nuevos con un cliente anterior. No se cambia el perfil global literal_unicode61_v1.
- OS puede variar texto: se conserva procedencia real y comparación con lector,
  sin prometer transcripción/orden universal.
- Gates sensibles a carga: preservar fallos/condiciones y ejecutar cadena completa
  sin compilación del host concurrente; investigar sin atribuir causas no probadas.

## Open Questions

Ninguna para este recorte. Alcance Excel y aceptación de calidad con modelos reales
siguen consultados por separado y no condicionan implementar esta dependencia5a.
