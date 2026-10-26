# Design

## Context

Ver proposal.md. navigation.ts ya define las siete formas canónicas, pero Home
usa variantes circle-check/rocket/circle-user y CSS agrega rellenos. Settings
recibe el tema desde App y localStorage conserva preferencias fuera de SQLite.

## Goals / Non-Goals

Centralizar forma/color y la preferencia de presentación en App, conservando
instancias de paneles. No modificar iconos personalizados, colores semánticos de
estados ni originales, SQLite, operaciones Rust o catálogos generados.

## Decisions

- SectionIcon reutiliza sectionInfo y AppIcon; todos los accesos de sección lo
  usan y heredan una variable compartida. Se eliminan las excepciones CSS de
  relleno. No cambiar las claves del catálogo persistente por similitud visual.
- Una preferencia local versionada guarda un #RRGGBB validado, azul #3183a6
  tomado del breadcrumb. App aplica variables derivadas a la raíz: acciones,
  iconos, foco, enlaces, selección y franja de sección. Se calcula contraste
  sRGB para adaptar extremos y elegir texto negro/blanco legible. Los colores
  semánticos y de entidades permanecen independientes.
- Selector propio sin dependencias: rueda con conic-gradient, pointer capture
  y semántica slider/teclado; saturación y luminosidad en ranges nativos y campo
  hexadecimal. Hue/Saturation/Lightness se conservan al ajustar grises o extremos.
  Sugerencias primero, Custom al final. No abrir selectores del SO ni transmitir
  contenido a servicios externos.
- Traducción en los puntos de presentación de Svelte/TS. No traducir IDs,
  payloads, claves de propiedades importadas ni resultados del usuario. Los
  errores del núcleo mantienen su detalle; las etiquetas que los rodean cambian.
  No incorporar un sistema multilingüe: el pedido establece inglés.

## Risks / Trade-offs

- Colores extremos → ajuste de tonos por tema y contraste de controles/enlaces;
  se muestra el hexadecimal elegido aunque el tono visible se adapte.
- Variantes CSS heredadas → reemplazar reglas que fijan azul decorativo y
  verificar estilo computado de los siete iconos y botones.
- Traducción amplia → revisar literales/textos, compilar y contrastar con copia
  de fuentes anterior; no hacer reemplazos dentro de datos o del contrato.
- La revisión headless no acredita IPC nativo → declarar su alcance; conservar
  la aplicación abierta sin operar datos personales ni pedir pantalla nativa.

## Migration Plan

No migración de workspace. La clave nueva sólo se escribe al elegir color y
tolera ausencia/corrupción/fallo. Eliminar la preferencia devuelve el azul.
Compilar frontend e inspeccionar componentes compilados con datos sintéticos,
temas y ancho estrecho; no agregar suites ni publicar.
