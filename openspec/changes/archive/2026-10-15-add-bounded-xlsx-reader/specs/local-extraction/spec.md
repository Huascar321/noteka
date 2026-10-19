# Spec Delta

## ADDED Requirements

### Requirement: Lector XLSX independiente y sin activación implícita

El sistema SHALL ofrecer un lector local aislado de XLSX Transitional mediante un protocolo privado acotado de bytes, sin rutas ni acceso al workspace. MUST distinguirlo de la extracción integrada: no añade extractor de cola, operaciones públicas, migraciones ni indexación y no se inicia por importar, abrir o leer datos.

#### Scenario: Lectura explícita offline
- **WHEN** se invoca el helper con un XLSX sintético admitido y red denegada
- **THEN** devuelve una representación determinista completa del subconjunto declarado, sin modificar archivos ni iniciar UI, modelos o conexiones

#### Scenario: Producto existente conservado
- **WHEN** se consulta el catálogo, abre un workspace v10 o se intenta encolar XLSX antes de su integración
- **THEN** continúan API v1/51 operaciones y los extractores UTF-8/PDF; XLSX no se anuncia como extractor integrado y Home sigue independiente

### Requirement: Contenedor XLSX verificable y cerrado

El lector MUST verificar ZIP, CRC, nombres únicos, estructura XML y relaciones internas antes de emitir éxito. SHALL rechazar cifrado, macros, rutas ambiguas, ZIP64/multidisco, métodos distintos de Stored/Deflate, XML con DTD y OOXML fuera del subconjunto declarado. No extrae entradas a disco ni resuelve entidades o relaciones externas.

#### Scenario: Contenedor malformado o ambiguo
- **WHEN** hay truncamiento, CRC incorrecto, entradas duplicadas/solapadas, rutas con traversal, encabezados inconsistentes o partes requeridas ausentes
- **THEN** falla de forma identificable sin representación parcial ni reparación tolerante

#### Scenario: Contenido activo o externo
- **WHEN** el paquete declara macros, cifrado, conexiones, enlaces externos o relaciones con TargetMode externo
- **THEN** se rechaza sin ejecutar, abrir ni descargar su contenido y sin devolver datos del archivo en errores

#### Scenario: XML hostil o variante no admitida
- **WHEN** hay DTD, entidades personalizadas, codificación no UTF-8, namespace Strict o contenido alternativo que podría sustituir celdas
- **THEN** se identifica formato no admitido o XML inválido, sin fallback con pérdida

### Requirement: Celdas XLSX tipadas sin pérdida numérica

El lector SHALL conservar hojas en orden, nombres/visibilidad, coordenadas dispersas, rangos declarados/observados, merges, sistema de fechas y estilos numéricos. MUST separar ausencia de valor, texto, número léxico, booleano, error y fecha ISO; conservar Unicode/espacios sin convertir números a f64, seriales a fechas ni inferir encabezados.

#### Scenario: Números fechas y valores vacíos
- **WHEN** hay decimales largos, exponentes, seriales de fecha, booleanos, errores, celdas sin valor y cadenas vacías
- **THEN** los lexemas numéricos permanecen exactos y cada tipo/ausencia se distingue con su estilo y sistema de fechas; no se presenta un serial como fecha calculada

#### Scenario: Cadenas y hojas dispersas
- **WHEN** hay shared strings, texto inline/rich-text, Unicode, hojas ocultas, merges y una celda XFD1048576
- **THEN** se conserva el texto lógico y los localizadores sin expandir la grilla o los merges; el rango observado se calcula de las celdas, no de la dimensión declarada

#### Scenario: Celdas o referencias incoherentes
- **WHEN** existen coordenadas duplicadas/desordenadas, filas incompatibles, índices de strings/estilos inválidos o tipos/contenido de celda no soportados
- **THEN** el libro se rechaza sin omitir esas celdas ni inventar valores

### Requirement: Fórmulas XLSX y caché diferenciadas

El lector MUST conservar separadamente la fórmula literal, su tipo normal/shared/array, referencia e índice compartido y el valor guardado opcional. SHALL validar grupos compartidos sin expandirlos ni evaluar fórmulas. El resultado explicita que la caché puede faltar o estar obsoleta y no reproduce cálculo ni formato visual de Excel.

#### Scenario: Fórmula con caché y sin caché
- **WHEN** una celda contiene fórmula con resultado guardado, sin resultado, o una fórmula array
- **THEN** conserva fórmula/metadatos y caché tipada opcional sin afirmar que esté recalculada o sea vigente

#### Scenario: Fórmula compartida
- **WHEN** un grupo tiene ancla y seguidores válidos o carece de ancla/rango coherente
- **THEN** el caso válido retiene metadatos originales sin fabricar fórmulas expandidas y el incoherente falla completo

#### Scenario: Fórmula no admitida
- **WHEN** se encuentra fórmula data-table u otro tipo no soportado
- **THEN** se devuelve una limitación explícita sin omitirla, ejecutarla o reemplazarla por su caché

### Requirement: Presupuestos del lector XLSX

El lector SHALL limitar fuente a 1 MiB, ZIP a 256 entradas/4 MiB por entrada/16 MiB expandido, XML a 100.000 nodos/profundidad 64, hojas a 32 y celdas totales a 10.000. MUST limitar cada texto/fórmula a 131.072 bytes, representación a 1 MiB y stdout a 8 MiB, sin truncar éxito. Otros límites estructurales se documentan y verifican.

#### Scenario: Fronteras y expansión
- **WHEN** una entrada, representación, estructura o conteo alcanza su límite o lo excede, incluido un ZIP comprimido pequeño de expansión excesiva
- **THEN** la frontera admitida se conserva y el exceso falla sin asignación proporcional a dimensiones declaradas ni resultado parcial

#### Scenario: Exceso de respuesta
- **WHEN** la representación completa excede el presupuesto o una respuesta no respeta versión/hash/esquema/límites
- **THEN** no se emite ni acepta como éxito completo y el motivo no contiene valores ni fórmulas del archivo

### Requirement: Vida acotada del helper de lectura

El helper MUST terminar tras una lectura, fallo, panic o exceso; limitar heap vivo solicitado a 128 MiB, desactivar core dumps y acotar CPU a 4 segundos y vida a 5 segundos desde inicio, incluso bloqueado en pipes o sin padre. SHALL permitir acortar, nunca extender, el presupuesto inicial; PDF conserva su contención y protocolo previos.

#### Scenario: Pipes y caída del consumidor
- **WHEN** stdin queda abierto/incompleto, stdout se bloquea o desaparece el consumidor
- **THEN** el helper termina dentro del presupuesto sin quedar residente ni emitir datos del archivo en stderr

#### Scenario: Fallos de memoria panic y presupuesto
- **WHEN** se excede heap/CPU/deadline o hay panic
- **THEN** el proceso termina con código identificable, sin core dump, log sensible ni afectar al cliente

#### Scenario: Lecturas concurrentes y regresión PDF
- **WHEN** se ejecutan lectores independientes simultáneos o se procesa el corpus PDF previo tras compartir el runtime
- **THEN** cada proceso conserva su presupuesto/resultado y PDF mantiene admisión, salida, límites y contención existentes
