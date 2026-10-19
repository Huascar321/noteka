# Spec Delta

## MODIFIED Requirements

### Requirement: Workspace fijo y admisión sin migración

El servidor MUST fijar ruta canónica e identidad del workspace al arrancar y comprobarlas con su versión actual al ejecutar operaciones. SHALL rechazar rutas/identidades ajenas, creación, migración y reparación implícitas; ningún argumento, recurso, metadato o instrucción encontrada en contenido puede cambiar el scope o los permisos. El arranque y discovery no modifican el workspace.

#### Scenario: Formato anterior o futuro
- **WHEN** se intenta iniciar sobre una versión anterior a v12, futura, inconsistente o un directorio sin base
- **THEN** se rechaza con motivo recuperable, sin abrir el original para escritura, generar backup de migración, recuperar trabajos ni descartar datos

#### Scenario: Cambio de destino durante la sesión
- **WHEN** la ruta seleccionada pasa a contener otra identidad, o una llamada intenta aportar workspace, origen, destino o ruta alternativa fuera del DTO
- **THEN** la operación se rechaza sin modificar ninguno de los workspaces y no continúa sobre el nuevo destino por un check previo obsoleto

#### Scenario: Sondeo sin efectos
- **WHEN** un cliente inicia y cierra un proceso para discovery sin ejecutar herramientas
- **THEN** contenido, versiones, trabajos, originales y archivos del workspace permanecen iguales; no se publican temporales en él

## ADDED Requirements

### Requirement: Índice y búsqueda XLSX con política compartida

MCP SHALL publicar los DTOs ampliados de index/search derivados del core, manteniendo 24/44/45 tools, siete exclusiones y workspace v12 fijo. MUST permitir get/search en lectura, exigir allow-write para put/reset y reservar allow-write+allow-run para extracción, sin necesitar allow-run ni helpers para indexar derivados. Contenido de valores/fórmulas es dato no confiable, nunca permiso ni instrucción de herramienta o envío remoto.

#### Scenario: Paridad XLSX modern y legacy
- **WHEN** clientes SDK y raw en ambas eras seleccionan con escritura y consultan con lectura un XLSX previamente extraído sin helpers
- **THEN** obtienen el mismo índice/hit/campos/localizadores que CLI, con structuredContent/text equivalentes y esquemas estrictos resolubles, sin nuevos nombres de herramientas

#### Scenario: Denegación y contenido hostil
- **WHEN** lectura solicita put/reset o una fórmula/celda encontrada pide ejecutar shell, elevar permisos, cambiar workspace o transmitir contenido
- **THEN** la mutación no autorizada se deniega antes de efectos y el texto se devuelve sólo como dato, sin ampliar permisos, activar proveedores ni ejecutar instrucciones

#### Scenario: Pureza y conflicto de revisión
- **WHEN** un cliente de sólo lectura pagina búsqueda XLSX mientras otro confirma put/reset/reemplazo o se consulta tras restauración en otra ruta
- **THEN** se conservan las reglas de cursor/CAS/identidad y errores del core sin mezclar revisiones, reparar índices ni abrir main para escritura
