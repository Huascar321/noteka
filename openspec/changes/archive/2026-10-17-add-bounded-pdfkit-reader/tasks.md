# Tasks

## 1. Baseline y corpus

- [x] 1.1 Preservar baseline durable, catálogos completos y alcance sin integración; verificar hashes de 522 archivos/16 binarios y describir límites en docs/pdfkit-reader.md.
- [x] 1.2 Crear generador propio y expectativas humanas anteriores al lector, con páginas Unicode, rotación, vacíos, imágenes, cifrado, framing y cuotas; verificar hashes/manifest y documentar matriz.

## 2. Protocolo y lector

- [x] 2.1 Implementar protocolo Rust privado estricto y ejemplo validador; verificar cabecera, hash, metadatos, páginas, límites y respuestas hostiles con tests debug/release y documentar contrato.
- [x] 2.2 Implementar helper Swift y preparación separada por perfil sin staging Tauri; verificar corpus real, UTF16 estricto, contención, pipes, flags y recolección de procesos por perfil; documentar comandos y ADR0021.

## 3. Integración y revisión

- [x] 3.1 Verificar regresión Rust apropiada por perfil, fmt/Clippy, preservación de contratos, fuente y binarios del producto; repetir gates si cambian artefactos medidos y registrar atribución histórica si permanecen idénticos.
- [x] 3.2 Completar matriz requisito/escenario, revisión de código/diseño, documentación y plan con resultados comprobados; verificar OpenSpec estricto, sin atribuir cierre de toda5a ni aceptación nativa/Home.

## Workflow follow-up

- Sincronizar únicamente este delta implementado/verificado, contrastar bloques no afectados y archivar este cambio cuando las seis tareas estén completas.
- Continuar por recortes independientes de5a, respetando el límite horario vigente; mantener Home activo9/17.
