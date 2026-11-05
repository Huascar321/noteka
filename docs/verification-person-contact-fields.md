# Verificación: campos locales de Person

El detalle de referencia inspeccionado en lectura muestra Title, Location,
Phone, Email, Instagram, LinkedIn, Twitter/X, Website e Interests. Los campos
locales nuevos se pueden editar en View details; no se copiaron valores reales
ni se importaron contactos. Los textos no se abren ni se envían automáticamente.
Los intereses son una lista de etiquetas individuales, incluso con comas.

La validación compartida admite textos opcionales de hasta 2.048 bytes sin
controles y hasta 32 intereses únicos de 128 bytes. `entity.get` devuelve los
campos; `home.query` no los incorpora a su proyección de galería. Al editar se
conservan el resto del payload y CAS; un payload futuro incompatible bloquea el
cambio sin descartar datos. No cambia el esquema SQLite. Clientes antiguos con
validación cerrada requieren el contrato actualizado para editar estas personas.

- Rust: 8 pruebas de productividad y 11 de Home aprobadas; la nueva comprueba
  límites, campos mal tipados, duplicados, CAS, `null`, preservación de datos
  futuros, versión SQLite y ausencia de contacto en la proyección Home.
- Chromium y WebKit: 18 checks por motor con datos sintéticos para editar,
  intereses con comas y duplicados, guardar, reabrir, Cancel/Discard, foco y
  420 px. Sin solicitudes externas. Goal detail pasó 28 checks Chromium tras el
  cambio; no se hizo aceptación nativa Tauri/macOS.
- Generación de contratos, build y validación estricta aprobados; Vite mantiene
  su aviso de chunk principal grande. Delta sin sync ni archivo.
