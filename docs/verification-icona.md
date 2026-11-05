# Verificación de Icona

Fecha:26/10/2026. Cambio `replace-ui-icons-with-icona`. Resultado local verificado;
delta pendiente de sincronización/archivo, sin publicación ni aceptación nativa.

Esta evidencia corresponde a la selección inicial de animaciones. La decisión
posterior de usar Jelly de Tasks en los nueve accesos tiene
[verificación propia](verification-sidebar-jelly.md), sin invalidar la revisión
inicial de geometría, imágenes o datos.

## Alcance y procedencia

85 claves conservadas:53 Fill y32 outline bold2.4. Revisión upstream
dd9f5217720a3f459a1f5464125701eb0d51c3a2;66 variantes exportadas,71 árboles
deduplicados tras cinco adaptaciones y nueve escenas CSS oficiales. Las figuras
sin equivalente exacto usan las [equivalencias documentadas](../third-party/icona/README.md).
Las imágenes originales, nombres persistidos, colores y datos se mantienen.

No cambia Rust, contratos, SQLite, backend o permisos. No se repitió la regresión
Rust de otros cambios: sus seis fallos previos siguen documentados en
[la verificación de page lock](verification-page-lock.md).

## Comprobaciones

- `npm run build`: icons/check, contratos/check, Svelte0 errores/advertencias y
  bundle de producción aprobado. Permanece el aviso previo de chunk mayor a500kB;
  no se cambió el umbral. Catálogo estático no importa motions ni React.
- Reimportación desde checkout limpio/revisión exacta y regeneración: assets,
  package metadata y ambos archivos generados idénticos byte a byte. SHA256 de
  assets en la procedencia. `icons:check` compara también avisos offline.
- Chromium y WebKit:13 comprobaciones de catálogo y60 de movimiento **por motor**.
  Los85 iconos se renderizan mediante AppIcon a18/24/48px en ambos temas,
  recortes/IDs únicos, peso efectivo por CSS, fallback desconocido/imagen ausente
  e imagen local válida. Cuatro capturas completas del catálogo revisadas.
- Movimiento: carga sólo al primer clic; hover/render estáticos; las nueve escenas
  CSS activas, iteraciones/duración finitas y restauración exacta del SVG y de los
  mismos nodos/anchors. Enter/Space, clic repetido, cancelación global, reduced
  motion antes/durante, hidden y desmontaje durante carga o reproducción aprobados.
  La visibilidad se controla por evento/propiedad en el navegador de prueba.
- UI de producción con puente CLI real y workspace temporal aislado:35 checks.
  Siete secciones conservan forma compartida en lateral/página y peso Fill/bold;
  clic real inicia una sola escena. Toggle de sidebar, menú, candado/open-lock,
  color personalizado, imagen original por attachment.read y guion blanco de Tasks
  verificados. Entidades/documentos/revisiones idénticos antes/después de navegar,
  leer y desbloquear temporalmente. Cero errores de página y pedidos externos.
- Layout estable: seis checks adicionales en claro/oscuro y1280/420px; siete
  iconos laterales visibles en escritorio y sin overflow horizontal. Las capturas
  esperan que finalice la transición de sidebar tras resize.

Total: **187 comprobaciones headless** focalizadas, además de build y generación.
No se añadieron tests que repliquen paths del catálogo ni se ampliaron suites
ajenas. La importación y revisión de recursos de terceros fue explícita;
la aplicación no descarga iconos ni mueve datos a servicios externos.

## Evidencia y límites

Artefactos ignorados bajo `output/playwright/icona-20261026/`: scripts, resultados
JSON de catálogo/movimiento por motor, `app-checks.json`, `layout-checks.json`,
cuatro catálogos y cuatro capturas de la nota. Servidor CLI confinado a su corpus
temporal; CLI macOS con red denegada. Harness de componentes usa mocks sólo para
sus dos imágenes; la UI de producción lee una imagen real del corpus por CLI.

Los primeros intentos fallidos del driver se conservan: workspace nuevo tratado
como existente, nombre de ruta Goals singular y locator Inbox ambiguo. Se
corrigieron fixtures/locators, sin modificar código del producto para hacerlos pasar.
Una captura intermedia de resize se reemplazó por layout asentado; la geometría
final del sidebar fue comprobada. No se atribuyen esos fallos al producto.

No acredita IPC/cierre de Tauri, percepción en la pantalla física ni todos los
sistemas operativos. Marca y controles nativos conservan su identidad. La licencia
upstream se declara en sitio/package; NOTICE distingue eso del texto MIT estándar
reproducido, ya que esa revisión no incluye LICENSE separado.
