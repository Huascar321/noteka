# Spec Delta

## Purpose

Gestionar reuniones locales con audio original, transcripción y hablantes
corregibles, conclusiones acreditadas y recuperación mediante operaciones Rust
compartidas por interfaz, CLI y MCP.

## ADDED Requirements

### Requirement: Inicio explícito y ciclo de captura

Meeting Notes SHALL preparar una reunión a demanda y proponer iniciar al detectar
una posible llamada. MUST capturar sólo después de una acción explícita Iniciar,
con fuentes visibles, permisos del SO, indicador y controles Pausar/Reanudar/
Finalizar. El sistema MUST NOT exigir una confirmación adicional por participante ni iniciar
al abrir una nota, workspace, popup o al recuperar una sesión interrumpida.

#### Scenario: Detección y decisión
- **WHEN** aparece una señal de posible reunión y el usuario rechaza el popup
- **THEN** no se abre micrófono, captura audio ni activa ASR; se evita insistir
  repetidamente por la misma señal y `/meet` permanece disponible

#### Scenario: Permiso o recurso no disponible
- **WHEN** Iniciar encuentra permisos denegados, fuente ausente o captura ya activa
- **THEN** informa el impedimento y conserva las notas, sin capturar otra fuente
  silenciosamente ni presentar una transcripción inexistente

### Requirement: Audio nativo y fuentes diferenciadas

La captura SHALL admitir micrófono y audio del sistema en macOS/Windows y guardar
pistas locales separadas con reloj/offsets verificables. MUST mostrar si el alcance
es aplicación o audio del equipo, conservar pausas/huecos y detener ambas fuentes
al pausar. La detección MUST NOT dar una llamada por finalizada sólo por silencio
o por micrófono muteado.
La ruta de ASR SHALL tratar eco/doble captura usando audio remoto como referencia,
conservando el audio original separado del preprocesado derivado.

#### Scenario: Fuente de una videollamada
- **WHEN** se inicia una reunión con auriculares y fuente seleccionada disponible
- **THEN** se obtiene el micrófono y el audio remoto directamente de la fuente,
  sin depender de que el altavoz vuelva a sonar por el micrófono

#### Scenario: Dispositivo o captura interrumpidos
- **WHEN** se desconecta la fuente o el proceso no puede conservar el audio
- **THEN** muestra estado interrumpido/atención y los huecos reales, conserva lo
  confirmado y no cambia a una fuente más amplia sin selección explícita

### Requirement: Runtime local y modelos identificados

ASR y diarización SHALL ejecutarse localmente en un supervisor acotado con
versiones, hashes y capacidades identificadas. El perfil inicial SHALL admitir
español, streaming y diarización con CPU común y aceleración disponible. MUST separar preparación de modelos del
inicio, impedir descargas implícitas y mantener las notas utilizables sin modelos.

#### Scenario: Preparación y funcionamiento offline
- **WHEN** los binarios/modelos acreditados están preparados y se inicia sin red
- **THEN** la captura y transcripción local funcionan sin servicios externos y
  el resto de Noteka no depende del runtime de reunión

#### Scenario: Modelo ausente o resultado tardío
- **WHEN** falta un modelo, cambia su huella o termina un worker obsoleto
- **THEN** se informa la capacidad real o se rechaza ese resultado; no se descarga
  al abrir ni se publica una generación incompatible como transcripción vigente

### Requirement: Transcripción temporal y atribución revisable

El núcleo SHALL conservar tramos con ID estable, offsets, fuente, texto original
y hablantes anónimos, distinguiendo parciales de finales. MUST documentar la
capacidad de hasta ocho voces automáticas del modelo propuesto, no atribuir nombres
reales por similitud de voz y permitir texto/actores corregidos, renombrar, agrupar
y reasignar tramos con CAS sin destruir la salida original.
Un nombre propuesto desde texto/contexto SHALL mostrarse como sugerencia con
fuente y requerir confirmación antes de asignar una identidad People.

#### Scenario: Hablantes y corrección
- **WHEN** se renombra Hablante 1 o se reasigna un tramo tras finalizar
- **THEN** las vistas resuelven la identidad estable y su vínculo opcional a
  People, conservan salida original/corrección y no crean personas por suposición

#### Scenario: Solapamiento o edición concurrente
- **WHEN** las voces se solapan, la atribución no es concluyente o cambia la revisión
- **THEN** no se inventa precisión/nombre y la corrección conflictiva se rechaza
  sin sobrescribir un cambio anterior; reprocesar crea otra generación revisable

### Requirement: Conclusiones con fuentes y acciones confirmadas

Al finalizar SHALL generarse conclusiones desde transcripción y notas si existe
un perfil LLM seleccionado, separando resumen, decisiones, preguntas y acciones.
MUST vincular afirmaciones a IDs/revisión de tramos y tiempos existentes, preservar
notas/correcciones y señalar resultados desactualizados. El uso remoto SHALL exigir
selección explícita para Meeting Notes; crear tareas SHALL requerir aceptar borradores.

#### Scenario: Finalización y fuentes
- **WHEN** finaliza la captura y hay transcripción y perfil válido
- **THEN** se genera un resultado revisable con navegación a sus fuentes sin
  convertir una cita estructuralmente válida en garantía de corrección semántica

#### Scenario: Perfil ausente y acción repetida
- **WHEN** no existe perfil, falla la generación o se repite aceptar una acción
- **THEN** audio/notas/transcripción permanecen, se ofrece la acción correspondiente
  y una confirmación idempotente no duplica tareas ni envía mensajes

### Requirement: Persistencia recuperable y límites

Rust/SQLite y audio original administrado SHALL ser fuente de verdad, con
sesiones/generaciones/revisiones y publicación posterior al audio durable. MUST
preservar documentos anteriores, usar migración aditiva desde la versión vigente,
CAS, lease con fencing y cuotas: una captura por host, ocho horas por sesión,
100.000 tramos, 32 MiB de texto y archivos dentro de 64 MiB existentes.

#### Scenario: Cierre o caída
- **WHEN** se interrumpe el proceso después de conservar audio y parte del texto,
  y el usuario solicita recuperar
- **THEN** se recupera lo acreditado con estado Interrumpida, originales y huecos
  explícitos, sin volver a encender captura o publicar resúmenes incompletos como finales
  ni convertir una lectura en una recuperación que modifique datos

#### Scenario: Duplicación y cuota
- **WHEN** dos clientes intentan iniciar, se repite un comando o se alcanza una cuota
- **THEN** sólo un propietario publica y se devuelve estado/idempotencia o error
  estructurado; no se borra audio previo ni se descartan tramos silenciosamente

### Requirement: Operaciones compartidas y biblioteca local

Las operaciones de sesión, estado, transcripción, actores y conclusiones SHALL
compartir validaciones Rust y catálogos UI/CLI/MCP. La biblioteca SHALL consultar
This Week/All/By People/Calendar con fecha civil del workspace, relaciones reales,
páginas 1–100 y cursores ligados al filtro. MUST abrir el original, paginar
lecturas sin cuerpos/audio completos y conservar acceso offline sin LLM.

#### Scenario: Semana y participantes
- **WHEN** se consulta una semana o persona vinculada a reuniones
- **THEN** se usan fecha de reunión y vínculos explícitos, no fecha de edición o
  nombres inferidos; las páginas conservan orden e identidades sin duplicados

#### Scenario: Lectura o cursor ajeno
- **WHEN** un cliente sólo lista/lee o usa un cursor de otro workspace/filtro
- **THEN** no activa captura/modelos, respeta las cuotas o rechaza la mezcla y no
  expone rutas arbitrarias, secretos ni contenido en logs de diagnóstico
