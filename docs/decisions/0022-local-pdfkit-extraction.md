# ADR 0022: integración explícita de páginas PDFKit

Estado: propuesto e implementación parcial; no adoptado como formato persistente.
Cambio activo add-local-pdfkit-extraction, 17 de octubre de 2026.

## Context

El lector independiente está cerrado. Integrarlo requiere una representación
recuperable distinta del PDF estricto, consumidor Rust y operaciones compartidas.
Baseline previo durable:542 fuentes/22 builds, SQLite13/API53/MCP25/45/46/batch9.

## Decision

Primero consumidor privado de bytes y fixtures13 auténticas, antes de DDL.
noteka-pdfkit-runtime supervisa el proceso explícito, deadline/cerca/pipes y toda
la respuesta. El error de cerca del caller se conserva; no publica generaciones
ni abre SQLite. Hijo con deny network, entorno mínimo y CWD temporal. Drop termina
y recolecta únicamente ese Child, sin matar procesos de otros trabajos.

Después se propone pdfkit_text como tipo explícito, Document v1 canónico completo
en chunks UTF-8 <=4096/512 y representación <=1MiB, retención mixta32MiB. Tablas014
aditivas, backup13 previo consistente, barrera14 y snapshots64MiB; originales y
base hash v2 intactos. No modificar001–013. Lectura pública propuesta
extraction.pdfkit_pages, sin helper para leer y sin FTS/UI hasta otro cambio.

No sustituir pdf_text: su perfil y semántica son distintos. Guardar sólo texto
eliminaría procedencia/limitaciones; duplicar texto en manifest rompería contabilidad.
Reutilizar offsets literales o PDF estricto fabricaría localizadores. JSON versionado
del perfil preserva páginas/estados/texto/metadatos sin claims visuales.

## Recovery

Antes de014 ya existen fixtures reales13, vacía/mixta, creadas con CLI13 durable.
La mixta tiene cuatro generaciones UTF8/PDF/XLSX/OCR y tres selecciones previas.
Backup y restore13 conservan exactamente schema/filas de39 tablas y originales.
Catálogo53 capturado. La prueba de datos/configuración desconocidos y migración14
todavía está pendiente; no se presenta una fixture SQL como histórico independiente.

Implementación futura debe verificar rollback antes/después de versión, cliente13
rechazando14, cliente14 rechazando15, backup13 restaurable en nueva ruta, cero
launches al migrar/restaurar y cuotas/ownership dentro del writer lock.

## Evidence

active-pdfkit-extraction-path.txt apunta a evidencia externa. Runtime/formato probados con
siete tests por perfil y22 PDF reales por perfil, incluida recarga canónica. No existe014 ni
cambio de versión/API/MCP todavía. La integración completa y adopción del formato
requieren tareas restantes y aceptación del cambio. Deadline5a permanece02:16UTC.
