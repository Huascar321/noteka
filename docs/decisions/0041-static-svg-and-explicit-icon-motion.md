# ADR0041 — SVG estáticos y movimiento explícito de iconos

Fecha: 23/10/2026. Estado: aceptada. Reemplaza la elección de catálogo de ADR0034;
los campos opcionales, validaciones e imágenes locales de ADR0034/0036 siguen vigentes.

## Contexto

El usuario pidió AnimateIcons, con formato optimizado y animación únicamente al
pulsar un botón lateral. AppIcon ya concentra el render de claves e imágenes.

## Decisión

Fijar `@animateicons/vanilla` 0.1.0 y generar sólo los 69 SVG seleccionados a partir
de sus datos y renderer oficial de reposo. Huge es la familia principal; los
faltantes usan Lucide del mismo proveedor. El runtime inicial recibe formas
estáticas locales, sin datos de movimiento ni catálogo completo. Las claves
previas y la rama de imágenes de entidad se conservan, sin escritura de datos.

Una acción Svelte únicamente en los ocho botones laterales importa el reproductor
y sus ocho definiciones al primer clic. No hay hover listeners. Mantiene un único
ciclo global de 800 ms; destruye controles y restaura todos los atributos originales.
Clic repetido, reducción de movimiento, visibilitychange y desmontaje cancelan la
reproducción o su carga pendiente. La navegación no depende de la decoración.

Los árboles animables mantienen la estructura del proveedor para corresponder
nodos con datos. El generador valida tags/atributos y referencias locales; los IDs
se aíslan por instancia. `icons:check` verifica la vigencia durante build. Se
conservan MIT y los avisos ISC/MIT de Lucide, Feather y Hugeicons.

## Alternativas y consecuencias

Importar un catálogo animado completo en AppIcon aumentaría trabajo inicial y
facilitaría movimiento involuntario. Un sprite raster perdería escalabilidad y
color del tema. Los SVG inline seleccionados permiten reposo inmediato y player
separado: el chunk de animación se mide en 74,69 kB (25,92 kB gzip) y se carga al
clic. El catálogo detallado aumenta el bundle principal respecto del anterior;
no se presenta esta entrega como una reducción de su tamaño total.

No cambia Rust, SQLite, API, datos ni permisos remotos. El movimiento se revisa
con UI de producción sobre copia del workspace; no acredita IPC/cierre Tauri.
