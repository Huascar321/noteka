# Design

## Context

Search tiene una variante verificada de `blur(6px) saturate(1.15)` y tinte gris con tinta propia. Los otros popups usan `.glass-menu`, `.glass-popup` o selectores del overlay anterior. Due separa material y contenido en hermanos, por lo que cambiar sólo la capa de vidrio no cambia la tinta del calendario.

## Goals / Non-Goals

**Goals:** una receta reutilizable para todo popup con vidrio, sin excepciones antiguas que tapen su transmisión.

**Non-Goals:** convertir documentos, navegación persistente o controles del sistema en popups; cambiar operaciones o animaciones; refracción nativa o transparencia al escritorio.

## Decisions

- Promover los tokens de Search a `--glass-overlay-*` y usarlos en los dueños de material existentes. No cambiar globalmente `--glass-surface`, también usado por cabeceras estáticas. Los alias históricos dejan de definir recetas independientes.
- Aplicar tinta principal/secundaria del overlay a todos los popups y al padre `.date-picker`, que contiene tanto material como números. Conservar señales semánticas y controles sólidos locales cuando corresponda a su función.
- Settings deja de superponer velos blancos en sidebar/main; las confirmaciones dejan de sobrescribir el tinte. Radios, bordes y sombras de menú/diálogo conservan jerarquía propia.
- La transición existente copia el material calculado, por lo que no necesita otro filtro ni cambios JavaScript. El selector de material respeta `.liquid-morph-active` y los fallbacks posteriores con especificidad suficiente.
- Refinamiento posterior del 10/11: el usuario pide blancos más luminosos. El tinte claro pasa de `#c2c2c2ad` a `#eeeeeead`, conservando alfa `ad`, blur/saturación, sheen y tinta. El tema oscuro conserva `#202020a8`. Es una afinación de la receta compartida, no otro material por componente.
- Rust y SQLite siguen siendo dueños de datos/reglas. No hay migración, dependencia ni cambios en guardado. Los cambios anteriores y los artefactos de Search se conservan como antecedente histórico.

## Risks / Trade-offs

- [Texto secundario/semántico pierde contraste] → medir superficies compuestas y conservar fondos locales semánticos; ajustar la tinta dentro del overlay si es necesario.
- [Una excepción de CSS conserva material viejo] → inventario de clases y estilos calculados, galería y recorridos de menús/pickers/guardias/Settings.
- [WebKit headless no compone el blur como Chromium] → conservar capturas por motor y no afirmar equivalencia nativa.

## Migration Plan

Build/HMR local. Reversión de reglas compartidas y documentación; no hay persistencia que migrar ni despliegue autorizado.
