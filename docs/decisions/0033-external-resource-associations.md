# ADR 0033: Asociaciones externas genéricas en SQLite

- Fecha: 22 de octubre de 2026
- Estado: adoptado para Google Drive y futuros proveedores
- Ámbito: vínculos entre entidades y recursos externos

## Contexto

Picker resuelve el acceso por archivo, pero Noteka necesita recordar qué archivo
eligió el usuario y a qué entidad pertenece. Guardar IDs en Keychain o en un JSON
fuera del workspace haría que la asociación no fuera parte de la fuente de verdad
local, no sobreviviera al backup/restore y no participara de la auditoría.

## Decisión

1. Migración020 agrega `external_resources` y `entity_external_resources`.
2. La identidad lógica de un recurso es
   `(provider, account_id, resource_type, resource_id)`; un mismo recurso puede
   asociarse a varias entidades sin duplicar metadata.
3. `external.link|unlink|list` son operaciones de Rust compartidas por CLI/API/MCP.
   `link` hace upsert de recurso y asociación, exige revisión de entidad y
   actualiza metadata; `unlink` quita sólo la asociación; `list` pagina.
4. Crear, actualizar o quitar una asociación incrementa la revisión de la entidad
   y agrega `external.link`/`external.unlink` en `changes`.
5. Desvincular no borra el recurso remoto ni la fila del recurso. El archivo de
   Drive sólo se modifica mediante una acción explícita futura, que este ADR no
   autoriza.
6. La metadata es JSON acotado a 64 KiB. No contiene tokens ni credenciales.
7. La UI básica consume las operaciones para vincular, listar, previsualizar,
   abrir y desvincular. El preview exporta Docs a texto y Sheets a CSV con la
   sesión Picker, no persiste contenido y falla explícitamente sin red o permiso.
   La edición y la réplica visual siguen fuera de este tramo.

## Alternativas

- Guardar sólo IDs en la sesión Picker: no es fuente de verdad, no sobrevive a
  backups y no permite listar asociaciones por entidad. Descartado.
- Agregar columnas por proveedor: contradice el modelo genérico y obliga a migrar
  por cada integración. Descartado.
- Borrar el recurso interno al quitar la última asociación: perdería metadata
  compartida y complicaría re-vincular. Se conserva como registro sin asociación.

## Consecuencias

- SQLite20; API v1/66; MCP 34/57/59 herramientas; batch9 sin cambios.
- El backup/restore incluye asociaciones porque viven en SQLite.
- La asociación no concede acceso por sí sola: el token Picker y `drive.file`
  siguen siendo la frontera de acceso.
- La metadata puede quedar desactualizada hasta que se refresque explícitamente;
  no se presenta como versión canónica del archivo.

## Evidencia

`docs/verification-external-resources.md`; pruebas de core y CLI real.
