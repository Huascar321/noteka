# Design

## Context

AppIcon centraliza iconos y ya conserva imágenes locales `image-UUID`. El usuario
eligió AnimateIcons y limitó las animaciones al clic lateral. La aplicación Svelte
puede usar el paquete vanilla sin React ni custom elements globales.

## Goals / Non-Goals

**Goals:** formas nuevas, SVG estáticos seleccionados, animación breve explícita,
carga diferida y compatibilidad visual/datos. **Non-Goals:** animar hover o pantallas,
cambiar arte importado, proveedores remotos, nuevas operaciones o migraciones.

## Decisions

1. Fijar `@animateicons/vanilla` 0.1.0. Generar únicamente SVG usados desde sus
   datos y restingAttributes oficiales, sin propiedades de movimiento en el bundle
   principal. Huge como familia principal y Lucide del mismo proveedor para
   equivalentes ausentes. Claves semánticas previas e icon picker compatibles.
2. SVG inline conserva currentColor, dimensiones y accesibilidad. Sólo markup
   generado de un catálogo confiable, validado por tags/atributos; IDs locales
   por instancia. Imágenes de entidad permanecen en la rama existente.
3. Una acción Svelte en los ocho botones laterales carga el player/definiciones
   seleccionadas al clic. No listeners hover ni animación desde AppIcon. Un ciclo
   acotado a menos de un segundo; destruir controles y restaurar atributos exactos.
   Repetición, reduced motion, visibilitychange y desmontaje cancelan el ciclo.
4. Generador y verificación de vigencia integrados en build, con licencias
   AnimateIcons MIT y avisos de sus fuentes. Se elimina la dependencia Svelte
   anterior si la auditoría confirma que no tiene consumidores restantes.
5. No cambia propiedad de datos ni API: Rust conserva reglas y SQLite/adjuntos
   conservan verdad. Sin migración. Copia aislada para inspección manual y build;
   sin suites no solicitadas ni escrituras de iconos en entidades.

## Risks / Trade-offs

- Carga diferida puede retrasar el primer ciclo: conservar icono estático mientras
  llega; cancelar si cambia contexto o preferencia.
- El player usa el árbol SVG oficial: preservar estructura para los iconos
  animables y restaurar atributos después de cada ciclo.
- SVG detallados pueden aumentar markup: seleccionar/deduplicar formas y medir
  chunks reales; no importar bibliotecas completas al runtime inicial.
