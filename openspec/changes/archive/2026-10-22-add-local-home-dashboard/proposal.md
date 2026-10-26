# Proposal

## Why

El núcleo ya conserva entidades, colecciones y referencias, pero la aplicación todavía no tiene el Home solicitado: trabajar exige recorrer pantallas separadas. Ahora que 3c está verificada y archivada, corresponde reunir datos reales en esa composición sin presentar funciones futuras ni una maqueta como paridad con Ultimate Brain.

## What Changes

- Añadir Home de ancho amplio, portada local azul/verde, icono de casa, navegación compacta y secciones **Tasks → Notes → Projects → Tags → Goals → People**, siguiendo `docs/home-reference.md` y preservando el acceso al editor y vistas existentes.
- Añadir consulta Rust paginada para las vistas locales de Tasks, Notes, Projects, Tags y People, con contexto, grupos y resúmenes calculados desde las entidades/relaciones originales. Publicarla por el catálogo compartido, Tauri y CLI; sin crear colecciones espejo ni copiar registros.
- Implementar Today por tarea pendiente con vencimiento civil **menor o igual** al día de referencia. No cambiar My Day por `planned_date` ni Overdue por `due_date < día`; las demás pestañas habilitadas tendrán reglas propias documentadas, no filtros atribuidos a Notion sin inspección.
- Presentar listas compactas, grupos plegables, tablero de estados de proyectos y galería textual de personas; abrir y crear originales mediante las guardias/captura existentes. Refresco, paginación, errores y cambios de workspace no deben sobrescribir buffers ni reutilizar proyecciones de otra vista.
- Ofrecer temas claro/oscuro con foco y contraste verificables, conservando oscuro como valor inicial. La franja de clima y Goals mantienen su posición, **identificadas como pendientes**, sin valores ficticios ni llamadas remotas.
- Registrar matriz de cobertura visual/funcional y evidencia sintética en la implementación. Este primer Home es utilizable y verificable, **no la copia completa** de todas las pestañas/funciones de las capturas.

## Non-goals

Modelo/progreso de Goals, clima/proveedor/caché remota, fotos/organización de contactos, favoritos, clips/voz/journal/meetings, recurrencias, Content, cumpleaños/check-ins/pipeline, drag-and-drop, vistas configurables por usuario, importación de Notion y cambios del formato persistente. Son entregas posteriores con contratos propios; no se activarán controles que aparenten implementarlas. Sin LLM, OAuth, descargas personales, publicación ni servicios externos.

## Capabilities

### New Capabilities

Ninguna. El Home amplía productividad y acceso de escritorio existentes, no introduce una segunda autoridad de datos.

### Modified Capabilities

- `productivity-workflows`: proyecciones paginadas compartidas de entidades/contexto para Home; semántica explícita de Today y resúmenes/grupos de proyectos y tags sin mutar originales.
- `desktop-shell`: composición Home, vistas funcionales y cobertura pendiente visible, navegación/captura protegida y temas accesibles offline.

## Impact

Núcleo: contratos/dispatcher y módulo de lectura de productividad; CLI: nueva consulta y paridad. Escritorio: `App.svelte`, nuevo panel Home/componentes de presentación y estilos/tema; se reutilizan operaciones de entidad, relaciones, guardias y editor, no se modifica el motor de fórmulas ni el nodo incrustado. Se extenderán pruebas Rust/CLI, contratos y recorrido nativo con datos sintéticos y red denegada.

No se propone migración: SQLite **v7**, documentos **v1/v2/v3** y backups siguen siendo la fuente de verdad. La selección de vistas/grupos es estado de presentación; el tema es preferencia local no secreta. Dependencias cumplidas: fases 0–3c y FTS. Goals, fotos y clima no bloquean este tramo, pero sí la fidelidad funcional completa de la referencia.
