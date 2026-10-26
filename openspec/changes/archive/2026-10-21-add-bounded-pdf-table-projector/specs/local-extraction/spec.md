# Spec Delta

## ADDED Requirements

### Requirement: Proyección independiente de tablas PDF desde evidencia OCR

El proyector SHALL organizar evidencia OCR validada de una página mediante rectángulo y columnas explícitos, agrupando filas por solapamiento vertical común. MUST conservar íntegra la evidencia y referenciar cada ordinal exactamente una vez en celda o fuera del rectángulo, sin inferir columnas/encabezados, separar palabras, filtrar confianza o presentar vigencia del original.

#### Scenario: Columnas filas y encabezado elegidos
- **WHEN** regiones dispersas de una página caben íntegramente en columnas y filas no ambiguas, con o sin fila de encabezado explícita
- **THEN** produce filas de arriba abajo y celdas ordenadas por columna, incluidas vacías; conserva texto, geometría, confianza, motor, página/hash y ordinales sin conversión numérica o escritura

#### Scenario: Regiones fuera y texto hostil
- **WHEN** hay encabezados repetidos, celdas con apariencia de instrucciones y regiones totalmente fuera del rectángulo
- **THEN** todo permanece como dato original referenciado sin ejecutar instrucciones ni descartar regiones exteriores; la elección de encabezado no elimina celdas

### Requirement: Ambigüedad y capacidad cerradas del proyector PDF

El proyector MUST rechazar regiones que crucen bordes del rectángulo/columnas, filas con solapamiento ambiguo, geometría degenerada y límites excedidos sin resultado parcial. SHALL admitir hasta512 regiones,32 columnas,64 filas y512 celdas con evidencia canónica<=1MiB y salida<=2MiB; no expande merges ni inventa texto fuera de las regiones.

#### Scenario: Cruces y agrupación ambigua
- **WHEN** una región cruza una columna/rectángulo o une filas sin intersección vertical común
- **THEN** falla completo con motivo identificable y sin escoger por centro, dividir texto, truncar o emitir una tabla aparentemente completa

#### Scenario: Entradas y fronteras
- **WHEN** se usan coordenadas/campos/perfil/ordinales inválidos, encabezado fuera de filas o límites en frontera/exceso
- **THEN** la frontera declarada se conserva y el exceso falla sin alterar la evidencia ni ampliar presupuestos

### Requirement: Proyector PDF sin activación del producto

La entrega SHALL ser una biblioteca local independiente y ejemplo de desarrollo con JSON acotado, sin acceso a workspace, archivos fuente, helpers, red o modelos. MUST conservar SQLite16/API56/MCP28/48/49/batch9 y Home; no constituye integración de tablas, vigencia comprobada de archivos, nueva operación pública ni cierre de5a.

#### Scenario: Evidencia sintética y PDF real controlado
- **WHEN** se proyecta golden independiente y evidencia obtenida por el OCR vigente de un PDF propio
- **THEN** se contrasta cada celda/ordinal y región exterior, se registran errores OCR reales y el producto conserva contratos/binarios/originales sin activación implícita

#### Scenario: Límite de autoridad
- **WHEN** se recibe una evidencia válida con hash declarado por el llamador
- **THEN** sólo se valida su estructura/proyección, sin atribuir verificación del original, fuente vigente o autorización para escritura/envío remoto
