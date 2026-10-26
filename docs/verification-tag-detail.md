# Revisión del detalle de Tag

Fecha: 24/10/2026. Cambio `match-tag-detail-reference`.

## Referencia y alcance

Playwright abrió el Tag vinculado al Project usado como referencia previa, lo
pasó a página completa y capturó su detalle en Notion. La identidad de origen
coincide con el Tag local; no se editó Notion. Las capturas y las identidades
personales quedan ignoradas y en evidencia privada, fuera de Git.

La referencia presenta portada, icono/título, Type/Favorite/Note Count, relaciones,
Nav y Notes/Clips. La portada local es decorativa; no se descarga la remota.
Se muestran relaciones locales acreditadas, sin reproducir una jerarquía ni
People/Goals derivados ausentes del modelo. List conserva el orden del contexto;
no promete las pestañas Recent/Fav./A-Z/By Tag/Voice de la fuente.

## Implementación y conservación

- Type conserva Area/Resource/Entity/Topic y un valor desconocido hasta una
  selección explícita. Favorite prepara el borrador; metadatos de lectura siguen
  en View details.
- Note Count viene de `home.query/tags_alphabetical`, dentro de sus primeros
  cien resultados. Ausencia no se transforma en cero; error ofrece reintento.
  El botón desplaza y enfoca Notes sin modificar el borrador.
- Notes/Clips usan el contexto existente y distinguen web_clip por su tipo local.
  Tasks conserva apertura/captura en un desplegable. Otras relaciones y su editor
  permanecen en cabecera. Cursor común y deduplicación por ID se conservan.
- Sin cambios Rust, contratos, SQLite, documentos, locks ni dependencias.
  Quince archivos protegidos, incluidas herramientas y detalles Note/Project/Task,
  conservaron SHA-256. CSS anterior permanece como prefijo completo.
- Backup consistente y baseline previos fuera de Git. Lectura canónica posterior:
  entidad completa, revisión 3, payload y documento idénticos a la lectura previa.

## Compilación y recorrido manual

`npm run build` aprobado: iconos/contratos y Svelte con cero errores y
advertencias; Vite conserva el aviso previo de bundle mayor a 500 kB.
Bundle aceptado `index-gMgTF-sP.js` y CSS `index-Nxfjeh5B.css`.

Chromium usó la interfaz compilada y puente CLI sobre copia restaurada.
Las mutaciones se rechazan antes del puente; no se intentó ninguna.

- Tag guardado: Resource, Favorite false, contador canónico 11, once Notes y
  ningún Clip en el contexto completo de doce registros.
- Enter en contador enfoca Notes. Type Entity/Favorite true prepara Cambios
  pendientes. Cancelar navegación conserva el borrador y deshabilita captura.
  Todos los borradores de demostración fueron descartados.
- Crear Note, Clip y Task conserva ContextLink al Tag; Clip preselecciona web_clip
  sin descargar ni extraer contenido.
- Paginación simulada del DTO: cuatro registros, cursor y luego el resto con
  un ID repetido; resultado once Notes únicas y cursor agotado. Fallos simulados
  independientes permiten reintento. Resumen ausente muestra No disponible;
  clasificación desconocida se conserva sin cambios pendientes. No se modificó
  SQLite para simular estos casos.
- Recursos externos es el único grupo aplicable. Space abre, Tab alcanza Cerrar,
  Escape cierra/devuelve foco y hover muestra tooltip. Abrir/cerrar no invocó
  operaciones adicionales.
- Regresión de apertura: Project conserva progreso Rust 70,2% (33/47) y su grupo
  Recursos externos; Note conserva propiedades y tres grupos de herramientas.
- Capturas finales claras/oscuras y 420 px. Ancho viewport/documento 420/420,
  una única barra de pestañas, sin alertas al terminar; vuelta a Tag Guardado.

El controlador inicial de simulaciones leía `response.ok` también para el IPC
sin respuesta de pending_edits. Eso generó un aviso de protección de cierre en
la previsualización. Se corrigió el controlador y se repitió apertura/capturas;
no se atribuye ese aviso al producto ni acredita IPC nativo.

## Límites

Sin suites nuevas ni ejecutadas, guardado real, exportación, proveedor, audio,
captura web o aceptación nativa. No se capturó/cerró Noteka nativo. La cifra Home
y las filas del contexto mantienen reglas de archivo distintas y cobertura
acotada; no se amplían cuotas para hacer coincidir ambas. Las simulaciones sólo
acreditan estados y navegación del frontend sobre DTOs controlados.

## Cierre

Dos requisitos añadidos sincronizados y contrastados: desktop-shell 63→65.
Los 63 bloques previos, otras dieciséis specs y ocho artefactos Meeting Notes
conservan sus bytes. Archivado 3/3 como
`2026-10-24-match-tag-detail-reference`; validación estricta final 18/18.
Meeting Notes continúa propuesto 0/19. Eco Armario y voicebot no se operaron.
