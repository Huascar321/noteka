# Design

## Context

Preparación/lector/búsqueda agrupada cerraron 9/9, 6/6 y 6/6 el 17/10, con baseline
v12/API v1/52, 3.116 checks headless y cuatro gates finales nuevos. OCR todavía no
existe en producto. `proposal.md` delimita el recorte. Baseline previo de 469
archivos/ocho binarios en la ruta externa indicada por `active-ocr-reader-path.txt`.

Investigación: Vision revision 3/fast local reconoció dos líneas en español/inglés
sin red, 80,87 ms de reconocimiento en el primer probe; una repetición tuvo RSS
máximo observado 39.518.208 bytes y 0,10 s wall. Son probes, no SLA/corpus. Accurate
con CPU explícita y el perfil accurate por defecto agotaron sus plazos; se
conservan sus fallos y no se usan como fallback. Se leyeron docs oficiales Apple
y SDK instalado; no hay modelos, permisos o proveedores externos asumidos.

## Goals / Non-Goals

**Goals:** lector verificable previo a persistencia, resultados falibles con
regiones/procedencia y contrato Rust que valide cada respuesta adversarial.

**Non-Goals:** ver propuesta; tampoco convertir imágenes a texto literal o usar
un corpus propio como garantía universal de reconocimiento. No se importa la UI
ni se modifica Tauri en este cambio.

## Decisions

1. Helper Swift nativo sobre Vision/ImageIO/CryptoKit ya disponibles en macOS;
protocolo Rust independiente sin enlazar Vision al core. Perfil mínimo macOS 13,
revision 3/fast y lista exacta es-ES/en-US. Consultar soporte del perfil y fallar
si falta. No `.accurate`, detección automática, customWords ni corrección. Texto
es el primer candidato de cada observación, en su orden, sin normalizar ni usar
confianza como probabilidad de verdad. Registrar OS y revisión/perfil exactos.
2. Entrada privada de 24 bytes con magic/version/longitud/plazo y bytes fuente,
sin rutas. Rechazar framing/EOF extra, tipo distinto de PNG/JPEG, fuentes múltiples,
dimensiones/área excesivas antes de decodificar píxeles, orientación inválida y
fallo de ImageIO. Handler recibe orientación 1–8; metadatos distinguen dimensiones
crudas y orientadas. Los rectángulos se refieren al marco orientado, no bytes o
regiones semánticas inferidas. Se conserva el hash de todos los bytes originales.
3. Respuesta JSON cerrada con perfil/OS, imagen y regiones. Rust valida rangos,
campos/errores, hash, ordinales contiguos, bbox/confianza finitos, dimensiones
congruentes y representación <=1 MiB. Máximo 512 observaciones/131.072 bytes cada
texto; no NUL, sin cortar o descartar regiones. Protocolo tipado separado permite
reutilizarlo al diseñar persistencia posterior, sin inventar tabla vacía ahora.
4. CPU 4 s/core dumps off antes de leer; watchdog independiente de stdin/stdout
con plazo inicial 5 s y sólo acortamiento. Rechazar máximo RSS observado >128 MiB
con muestreo de 5 ms. Esto **no** sustituye un allocator instrumentado ni garantiza
límite duro RSS o GPU: allocations de frameworks no pasan por el allocator Rust.
Los caps exactos previos de PDF/XLSX permanecen sin cambios; las mediciones OCR se
etiquetan por método. Los errores son enums fijos; no imprimir texto/errores del
framework. El runner de prueba acota pipes, mata y recolecta, sin un supervisor de
workspace fingido. La integración siguiente deberá reutilizar fencing/deadline y
diseñar recuperación antes de añadir formatos persistentes.
5. Preparador local usa SDK/Swift instalado y caché de módulos externa; no descarga
ni instala nada y falla explícitamente en plataforma/toolchain no disponibles.
Binarios en target ignorado, sin añadirlos a hooks o empaquetado Tauri todavía.
Corpus propio MIT generado sin pantalla con CoreText: textos esperados declarados
antes, PNG/JPEG, orientación/estilos, blancos y entradas inválidas. Medir CER/WER
por fixture y agregado; calidad separada de la validación exacta del protocolo.
No editar el esperado para hacerlo coincidir con el OCR. El perfil inicial no
promete manuscritos, fotos complejas, orden de lectura o tablas.
6. Tests Rust mutan toda la respuesta; frontera/exceso, framing y geometría.
Pruebas reales del helper con red denegada y timeout externo, bloqueos pipes,
presupuesto menor/CPU/RSS donde reproducible. Corpus/render no cuentan como UI
nativa. Fmt/Clippy/workspace, catálogos y frontend se verifican. Reusar regresión
costosa/gates recientes sólo con igualdad de core/binarios/corpus/gates; registrar
la adición aislada al workspace/lock, sin fingir nuevas corridas. Si cambian o
surge una duda material, repetir el check afectado antes del cierre.

## Alternatives

- Tesseract: no hay instalación verificada en las rutas habituales; añadir binario
y pesos contradiría la prohibición vigente de instalar modelos. No se instala.
- Vision accurate: no pasó los probes con presupuesto; no se activa por defecto.
- Vision dentro del host: rechazada por bloqueo/contención y separación de reglas.
- OCR como texto UTF-8 original o PDF: rechazado por procedencia/semántica incorrectas.

## Risks / Trade-offs

- [Dependencia OS] → backend explícito macOS y versión/procedencia; sin garantía de
otros sistemas o resultados idénticos entre OS, ni descarga para reparar soporte.
- [Reconocimiento falible] → calidad medida, confianza cruda, cero detecciones no
equivale a ausencia; nunca reemplaza original ni da contenido por cierto.
- [Memoria de frameworks] → área/bytes/regiones y vigilancia de RSS, medición
explícita, sin atribuirle el allocator Rust. No cambia las cuotas existentes.
- [Salida o entrada bloqueada] → watchdog independiente más timeout/reap de prueba.

## Migration Plan

Sin migración ni operación nueva. El lector/protocolo pueden retirarse sin tocar
workspaces. La integración siguiente deberá diseñar almacenamiento, recuperación,
lectura, indexación explícita y compatibilidad/catálogos; no se declara resuelta
por este helper. Documentar ADR de backend y evidencia antes de sincronizar sólo
los seis requisitos probados y archivar este cambio. Home permanece activo 9/17.
