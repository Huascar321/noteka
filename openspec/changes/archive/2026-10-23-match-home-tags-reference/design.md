# Design

## Context

Tags usa filas genéricas y tiene pocas relaciones de notas importadas. El export contiene quince tags, trece no archivados. La referencia muestra Area/Resource/Entity y última nota/contador, que deben provenir de entidades locales. El checkout y los ajustes anteriores se conservan.

## Goals / Non-Goals

**Goals:** cuatro vistas, grupos y filas fieles a la referencia, arte acreditado, recuperación de relaciones y resúmenes calculados.

**Non-Goals:** jerarquías completas de tags, fórmula universal Latest Activity, sincronización continua, dominio Goals, nuevos cuerpos o contadores copiados.

## Decisions

1. HomeTagSection/HomeTagRow comparten el encabezado y toolbar de Home. Types inicial, grupos Area naranja/Resource violeta/Entity verde; Topic independiente sólo cuando hay registros. No se presenta contexto de proyectos adicional en estas filas. Se mantienen apertura, captura y guardias existentes.
2. Classification añade Entity, sin renombrar Topic. TagFields añade favorite, source reutilizando NoteSource, y source_activity_at opcional RFC3339. Validación y filtros generales en Rust; ningún DDL. Detalle permite favorito/clasificación y conserva la procedencia/actividad de lectura.
3. Home filtra archivo de origen antes de paginar y añade TagsFavorites. Recent combina source_activity_at con el máximo timestamp de sus notas relacionadas no archivadas, antes de paginar; la modificación local sirve de fallback; el snapshot de actividad importada se declara explícitamente. A-Z y Types mantienen orden por título y desempate por ID.
4. Contador y última nota se calculan con relaciones Note→Tag existentes, únicas, activas localmente y no archivadas en procedencia. La última nota usa source.updated_at validado si existe o actualización local; interpreta RFC3339 como instante y desambigua por ID. No se cuenta por títulos ni se almacena una última nota ficticia.
5. El helper de recuperación verifica CSV/manifiesto y propiedades iniciales de cada página de tag. Las notas ya conservan IDs originales; sólo éstos acreditan enlaces. Dry-run predeterminado, backup nuevo, CAS e informe privado incremental; comparar IDs y digest documental antes/después. Arte desde URLs observadas, convertido a PNG cuando hace falta y guardado como adjunto local; conservar iconos elegidos.

## Risks / Trade-offs

- [Actividad y fechas exportadas son snapshots] → documentar procedencia y alcance, comparar con la referencia y corregir sólo metadata acreditada; no prometer actualizaciones remotas automáticas.
- [Referencias faltantes o ambiguas] → omitir e informar; nunca crear notas incompletas ni copiar el contador.
- [Cliente anterior cerrado] → actualizar antes de editar payloads enriquecidos; mantener backup e informe de reversión por revisión.
- [Aplicación parcial] → operaciones idempotentes e informe incremental; no afirmar atomicidad global ni restaurar sobre cambios posteriores.

## Migration Plan

Compilar Rust/contratos/UI, revisar dry-run, respaldar y aplicar metadata/vínculos acreditados. Inspeccionar manualmente las vistas, temas y ventana estrecha con UI de producción y copia real. Sincronizar sólo lo implementado después de contrastar código y requisitos. No ejecutar suites no solicitadas.
