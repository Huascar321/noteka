# Matriz del lector OCR independiente

Seis requisitos/catorce escenarios de `add-bounded-macos-ocr-reader`.
[Resultados y límites](verification-ocr-reader.md). No es evidencia de OCR
persistido, UI, búsqueda o acceso a proveedores.

| Escenario | Código/contrato | Evidencia |
| --- | --- | --- |
| Imagen admitida sin red | Perfil fijo Vision e ImageIO, protocolo Rust | 16 fixtures de texto PNG/JPEG y dos blancos, ambos perfiles, red denegada; hash/dimensiones/OS/perfil validados en Rust. |
| Formato dimensiones o secuencia no admitidos | Preflight antes de píxeles; header limitado | TIFF <=1 MiB, APNG de dos frames, eje >4096 y área >4M; fuente excesiva y entradas corruptas se rechazan sin truncar. |
| Perfil no disponible | Predicado de revisión/idiomas y error cerrado | Perfil real disponible comprobado; probe prueba revisión/idiomas ausentes y Rust acepta sólo error tipado. No se finge otro OS real. |
| Respuesta y origen ligados | Reply::decode y expected SHA-256 | Cada fixture real pasa por ejemplo Rust independiente; tests cambian hash/identidad y rechazan toda la respuesta. |
| Respuesta adulterada | Validación cerrada de metadatos/regiones | Tests mutan campos/ordinal/dimensiones/perfil/OS/geometría/confianza, no finitos y un millón de separadores; desconocidos por cada nivel rechazados. |
| Framing o error inválidos | Header/EOF y enum de errores | Diez casos reales malformados, magic/reservados/plazo/cuerpo/extra; unit JSON truncado/extra y error que intenta incluir contenido. |
| Texto orientación y geometría | Marco orientado y top candidate sin corrección | Ocho EXIF JPEG y textos con acentos/markup/serif/Courier; texto/errores y CER/WER íntegros, sin ejecutar contenido o ajustar esperado. |
| Imagen sin detecciones | Regiones vacías válidas con limitaciones | PNG/JPEG blancos reales y test de representación vacía; no se atribuye ausencia universal al resultado. |
| Frontera y exceso del resultado | Representation.validate sin recortar | Tests 512/513 regiones, 131072/+1 bytes UTF-8 y representación JSON exactamente 1 MiB/+1; todos los excesos rechazan completos. |
| Salida excesiva o desconocida | Consumidor con selectors/cupos/reap | Procesos sintéticos exceden stdout/stderr y timeout; Rust rechaza JSON/campos/errores desconocidos sin usar texto parcial. |
| Deadline bloqueo y presupuesto menor | Thread watchdog y min del plazo | Helper normal stdin bloqueado, header real 100 ms con cuerpo bloqueado, probes sleep/short/stdout; procesos terminados y recolectados, sin extender cinco segundos. |
| CPU memoria y fallo interno | setrlimit/core off/getrusage muestreado | Probes verifican límites OS, SIGXCPU/SIGKILL/watchdog, exceso RSS y fallo interno; diagnostics descartados, sin core dumps. Método RSS y tolerancias explícitos. |
| Lectura sin workspace | Bytes/canales privados, sin core enlazado | Corpus/procesos con cwd de desarrollo y fixtures externas, sin workspace/account/operaciones; worker no recibe rutas o credenciales. |
| Regresión de contratos y Home | Core/host/migraciones sin cambios | 298 Rust por perfil y once del protocolo finales repetidos, catálogos completos y ocho hashes iguales; 75 cliente/11 puente nuevos en esta aceptación. Reutilización delimitada de UI/oracles/gates previos con pins, sin fingir nuevas ejecuciones. |

La tolerancia de CPU medida de 4,5 s comprueba un límite OS configurado en 4 s;
no lo aumenta. RSS >128 MiB se detecta por muestreo, puede sobrepasarlo entre
muestras y no contabiliza GPU. No se atribuye el allocator Rust a frameworks.
