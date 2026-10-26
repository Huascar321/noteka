# Design

## Context

People lateral sigue usando el listado genérico. HomePersonSection ofrece siete
vistas Rust, fotos locales, calendario mensual y proyectos activos por persona.
Las páginas Tasks/Notes/Projects/Tags/Goals ya comparten layout y guardias.

## Goals / Non-Goals

**Goals:** composición People de referencia reutilizando datos y comportamiento.

**Non-Goals:** modificar contactos/importaciones, calendario externo, sincronización,
audio/transcripción o simular Meeting Notes, que requiere su propio cambio.

## Decisions

- PeoplePage monta HomePersonSection en modo página con HomeReader Contacts de
  diez filas, acumulación, cola de lectura y descarte por lifetime. El modo Home
  conserva encabezado compacto. No duplicar reglas o tarjetas en la interfaz.
- Nav selecciona Contacts/Pipeline/Birthdays, con Cal identificado como cumpleaños;
  el menú expandible ofrece todas las vistas. New Person conserva captura existente.
- WorkspacePane deriva People a su página, evita cargar el listado genérico y
  delega refresh. Breadcrumb y Guardar/Descartar/Cancelar son los compartidos.
- CSS sólo añade reglas de página; núcleo/contratos/SQLite no cambian. Alternativa
  descartada: recrear filtros/listados en Svelte, por riesgo de reglas divergentes.

## Risks / Trade-offs

- Cal no es un calendario general: nombre accesible y guía explican cumpleaños.
- Galería/fotos asíncronas: conservar fallbacks y límites existentes; revisión
  manual en copia restaurada y ambos temas/ancho estrecho, sin ejecutar suites.
- Meeting Notes tendrá proyecciones y captura propias; su investigación/diseño
  no se presenta como una función implementada por este cambio.
