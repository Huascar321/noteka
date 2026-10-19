# Spec Delta

## MODIFIED Requirements

### Requirement: Lector XLSX independiente y sin activación implícita

El sistema SHALL ofrecer un lector local aislado de XLSX Transitional mediante un protocolo privado acotado de bytes, sin rutas ni acceso al workspace. MUST separar ese helper de las operaciones del core: importar, abrir, leer, encolar, indexar, buscar o migrar no inicia parsing; sólo `extraction.run` explícito sobre un trabajo admitido lo ejecuta. Index/search usan derivados ya confirmados, no el contenedor XLSX.

#### Scenario: Lectura explícita offline
- **WHEN** se invoca el helper con un XLSX sintético admitido y red denegada
- **THEN** devuelve una representación determinista completa del subconjunto declarado, sin modificar archivos ni iniciar UI, modelos o conexiones

#### Scenario: Producto existente conservado
- **WHEN** se consulta el catálogo, abre un workspace actual o importa/encola/lee/indexa/busca XLSX sin ejecutar run
- **THEN** el catálogo y los contratos vigentes declarados en core-operations, incluido extraction.xlsx_cells, se conservan; index/search sólo evolucionan mediante cambios verificados y UTF-8/PDF/Home mantienen su comportamiento sin iniciar un helper

### Requirement: OCR aislado de fuentes y operaciones persistentes

El lector independiente MUST recibir bytes y no abrir workspaces ni recibir rutas originales, credenciales o configuración de proveedores. SHALL preservar formatos persistentes, catálogo/MCP vigentes, operaciones, fuentes, cuotas/validación/gates y Home, sin integrar procesamiento, indexación o reparación al montar o buscar; la integración persistente exige otro cambio verificado.

#### Scenario: Lectura sin workspace
- **WHEN** se ejecuta el lector sobre bytes sintéticos sin cuenta ni workspace
- **THEN** funciona de forma independiente sin crear tablas, jobs, índices o modificaciones de originales

#### Scenario: Regresión de contratos y Home
- **WHEN** se incorporan el protocolo/helper y se verifican las operaciones previas
- **THEN** catálogo, formatos persistentes, whitelist batch, lectores, búsqueda y guardias/Home conservan sus contratos y evidencia delimitada sin atribuir a la adición del lector una integración persistente nueva ni toda 5a completa

### Requirement: Presupuestos explícitos y errores útiles

La cola SHALL publicar y verificar límites de fuentes, jobs, fragmentos, almacenamiento derivado, páginas, respuestas y ejecución. MUST rechazar exceso antes de publicar, sin truncar un éxito como completo ni exponer contenido íntegro en errores. No sube los límites vigentes de importación, transporte o batches para acomodar derivados. Los 32 MiB suman texto UTF-8/PDF y JSON canónico XLSX/OCR.

#### Scenario: Fronteras y backpressure
- **WHEN** se alcanza un límite de 1 MiB de fuente, 1.000 queued/running, 10.000 trabajos o 32 MiB de texto derivado retenido
- **THEN** la frontera permitida se acepta y el exceso produce un motivo explícito, sin crear trabajos o resultados parciales

#### Scenario: Respuesta limitada
- **WHEN** una lectura pide un límite inválido o una respuesta completa supera 2 MiB
- **THEN** se rechaza o pagina conforme al contrato, sin ocultar cobertura, cursores, texto omitido o errores

#### Scenario: Límites PDF y presupuesto mixto
- **WHEN** un PDF excede 100 páginas, 1 MiB de texto derivado, 512 fragmentos, 4.096 bytes por fragmento o el presupuesto retenido junto con texto UTF-8
- **THEN** la frontera válida se acepta y el exceso rechaza la generación sin purga ni truncamiento; la regeneración descuenta sólo su resultado previo reemplazable y no usa el tamaño comprimido del PDF para contar texto

## ADDED Requirements

### Requirement: Lector PDF nativo independiente y explícito

Un lector descartable SHALL ofrecer el perfil local apple_pdfkit_text_v1 mediante bytes y un protocolo privado propio. MUST devolver páginas de texto derivado con hash fuente y procedencia OS/build sin modificar originales, abrir workspaces o activar operaciones del núcleo. Su incorporación no sustituye pdf_text ni inicia parsing al importar, leer, buscar o abrir la interfaz.

#### Scenario: Texto moderno offline
- **WHEN** se envía un PDF propio admitido con texto seleccionable Unicode y el perfil disponible, con red denegada
- **THEN** devuelve todas sus páginas en orden de documento y texto del API conservado, sin UI, modelos, proveedores o cambios de archivos

#### Scenario: Página sin texto seleccionable
- **WHEN** una página no devuelve texto, incluida una página escaneada
- **THEN** se conserva su posición y estado no_text con texto vacío, sin OCR ni prueba de ausencia visual

### Requirement: Admisión nativa y límites de fidelidad declarados

El lector MUST verificar framing PDF de cabecera/EOF, rechazar cifrado reconocido incluso desbloqueado y conservar estrictamente el texto devuelto por el API al convertir UTF-16 a UTF-8, sin normalización propia. SHALL declarar parser de sistema tolerante, sin validación completa de PDF, orden visual, OCR, tablas o geometría; no repara el original ni pide contraseñas.

#### Scenario: Cabecera EOF y cifrado
- **WHEN** falta framing admitido, el backend no reconoce el documento o isEncrypted/isLocked son verdaderos
- **THEN** falla con razón tipada sin texto parcial ni intento propio de desbloqueo, incluso si el sistema abrió una contraseña vacía

#### Scenario: Codificación y contenido como datos
- **WHEN** el texto del API tiene UTF-16 no convertible íntegramente, NUL, enlaces o contenido semejante a instrucciones
- **THEN** la codificación no válida se rechaza sin reemplazos; el contenido válido permanece dato sin ejecutar acciones ni abrir enlaces

#### Scenario: Tolerancia del sistema sin falsa validación
- **WHEN** el backend admite anomalías internas que el framing no detecta o el orden de texto difiere del visual
- **THEN** el resultado identifica esas limitaciones y el perfil/OS reales, sin afirmar integridad completa o modificar bytes fuente

### Requirement: Protocolo PDF nativo validado y acotado

El consumidor Rust SHALL verificar versión/magic/reservados, hash esperado, perfil, metadatos acotados y todas las páginas/estados/conteos de una respuesta. MUST limitar fuente y texto UTF-8 total a 1 MiB, páginas a 1–100, stdout a 8 MiB y stderr a 8 KiB, rechazando campos desconocidos, incoherencia o exceso completos sin fallback, omisión o truncamiento.

#### Scenario: Respuesta fuente y páginas ligadas
- **WHEN** la respuesta válida corresponde al hash de entrada y respeta límites
- **THEN** se admiten páginas consecutivas 1-based, text/no_text coherentes, bytes totales exactos y metadatos/procedencia completos

#### Scenario: Alteración y frontera
- **WHEN** hay versión/hash/perfil/metadata/ordinal/estado/count ajenos o se alcanza/excede una frontera de fuente/texto/páginas
- **THEN** la frontera válida se conserva y el caso inválido falla completo sin confiar en un exit code exitoso ni devolver datos parciales

#### Scenario: Pipes o envelope no admitidos
- **WHEN** hay JSON/framing inválido, bytes adicionales, protocolo de otro helper o pipes excesivos
- **THEN** se rechaza el resultado, termina/recolecta el proceso y no se reflejan datos de la fuente en el error

### Requirement: Contención nativa de PDF descartable

El helper MUST desactivar core dumps, limitar CPU a 4 s y vida a 5 s desde inicio aun bloqueado o sin consumidor; un plazo recibido sólo se acorta. SHALL terminar al observar máximo RSS mayor a 128 MiB, con muestreo de 5 ms y comprobaciones al publicar, declarando que no es límite duro RSS/GPU ni allocator Rust. Diagnósticos del framework no contaminan el transporte.

#### Scenario: Bloqueo y pérdida del consumidor
- **WHEN** stdin/stdout quedan bloqueados o desaparece el padre
- **THEN** el helper termina dentro del plazo/tolerancia declarados sin quedar residente ni producir core dump

#### Scenario: CPU memoria fallo y plazo menor
- **WHEN** se excede CPU/RSS observado, ocurre abort o se recibe un presupuesto menor
- **THEN** termina con estado identificable y sin extender el presupuesto, publicar texto parcial o afectar al cliente

#### Scenario: Controles separados del binario normal
- **WHEN** se ejecutan probes de contención o se envían sus flags al binario normal
- **THEN** los controles existen sólo en el build de pruebas y el normal los rechaza sin ejecutar sus acciones

### Requirement: Entrega nativa aislada y compatibilidad preservada

El lector SHALL prepararse por target/perfil macOS con SDK instalado, sin descargas ni staging automático del host. MUST conservar núcleo/UI, catálogo y formatos vigentes, lectores previos, cuotas/gates y Home; no añade perfil público o persistencia. Integrar resultados exige otro cambio con recuperación diseñada antes de alterar formatos.

#### Scenario: Preparación y disponibilidad
- **WHEN** se prepara para un target/perfil admitido o falta el SDK/plataforma
- **THEN** se obtiene el helper propio correspondiente o un error explícito, sin instalar recursos, fallback o ventana nativa

#### Scenario: Producto y evidencia previos
- **WHEN** se verifican las operaciones y artefactos existentes tras incorporar el lector independiente
- **THEN** conservan contratos y comportamiento medidos con procedencia de binarios real; no se declara PDF integrado o toda 5a completa por pruebas del lector
