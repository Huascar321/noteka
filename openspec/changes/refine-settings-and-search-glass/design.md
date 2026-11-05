# Design

## Context

Settings tiene fondo de diálogo y navegación opacos. Search usa tinte claro al 72% más un radial blanco de hasta 72%, y su input combina padding del marco y del propio input. Ver `proposal.md` para la motivación y `docs/design-system.md` para el material vigente.

## Goals / Non-Goals

**Goals:** hacer perceptible el vidrio sobre contenido real y unificar la búsqueda compacta sin degradar lectura o comportamiento.

**Non-Goals:** transparencia de ventana al escritorio, refracción nativa, imágenes de fondo decorativas, cambios en operaciones, persistencia o geometría general de los diálogos.

## Decisions

- Un material compartido de popup con tinte más transparente y sheen tenue reemplaza la acumulación del buscador. Un único filtro acotado vive en cada diálogo montado; los hijos sólo agregan velos de lectura sin más filtros. Se descarta reducir alfa sin proteger textos o filtrar todo el backdrop.
- Search usa velos ligeros en resultados, preview y controles. Settings comparte el marco y usa un velo de contenido un poco más denso, con navegación de vidrio y selección de cápsula. Se actualiza la guía: Settings es una superficie flotante con lectura protegida, no un cuerpo documental.
- Ambos campos pasan a 32 px, padding único y tipografía de 13 px como Search lateral, con placeholder Search y los aria-label existentes. Los iconos conservan tamaño compacto y cierre accesible; el foco no altera la geometría.
- El contraste se mide sobre píxeles compuestos, incluyendo fondos de extremos y color. Se comprueba transmisión de un fondo sintético de prueba y apariencia sobre Home real, sin agregar decoración al producto. El alfa exacto puede afinarse según esa medición.
- Reutilizar bordes enmascarados/reflejos y tokens existentes; fallbacks posteriores retiran las capas y filtros. Rust y SQLite conservan propiedad de datos y reglas; no se necesita migración ni dependencia.

## Risks / Trade-offs

- [Menos tinte reduce contraste] → tinta específica de popup y velos locales de lectura; medir claro/oscuro sobre fondos variables.
- [WebKit compone distinto] → comprobar píxeles y funciones en ambos motores, sin atribuir aceptación nativa al puente CLI.
- [Settings tiene área amplia] → sólo un blur local mientras está abierto; sin afirmación de costo GPU o equivalencia nativa.

## Migration Plan

Aplicación local por build/HMR. Se preservan los cambios anteriores y el checkpoint existente; con commit local conjunto autorizado posteriormente, sin push, sync ni archivo automático.
