# Tasks

## 1. Persistence and recovery

- [x] 1.1 Añadir `page_locked` opcional a NoteFields, regenerar contrato y verificar tipo/ausencia/null, CAS, preservación de documento/metadatos desconocidos y backup/restore/export con pruebas Rust; conservar SQLite/API/operaciones y ADR0054.

## 2. Desktop interaction

- [x] 2.1 Implementar Lock page, Locked/Re-lock y apertura temporal por instancia; proteger controles/handlers de edición y usar guardias sin perder borrador. Verificar build y recorridos headless del ciclo, campos, atajos y preservación (`block-editor`).

## 3. Verification and documentation

- [x] 3.1 Verificar errores/CAS, Guardar/Descartar/Cancelar, reapertura, teclado, claro/oscuro/420px y compatibilidad de tipos anteriores sobre SQLite sintético, sin red externa ni pantalla nativa; registrar resultados y límites.
- [x] 3.2 Actualizar guía/plan/contexto, contrastar las dos specs con código/evidencia y validar OpenSpec; dejar el cambio listo para sync/archivo sin alterar cambios ajenos.
