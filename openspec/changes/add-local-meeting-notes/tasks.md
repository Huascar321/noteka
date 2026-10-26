# Tasks

Estado: **sólo diseño/modelos autorizados**. Ninguna tarea siguiente se ejecuta
en esta entrega ni se marca completada por validar documentos.

## 1. Dominio y recuperación

- [ ] 1.1 Concretar contratos/tablas/manifiesto de audio, operaciones y estados propuestos; reconfirmar migración disponible, cuotas y recuperación antes de DDL (`meeting-notes`, `local-workspace`).
- [ ] 1.2 Implementar migración aditiva desde la versión vigente, sesión/nota canónica, tramos originales/correcciones/generaciones, CAS/lease/fencing y barrera de backup; preservar IDs/documentos/desconocidos sin convertir notas por título (`meeting-notes`, `local-workspace`).
- [ ] 1.3 Exponer operaciones compartidas, CLI/SDK/MCP y catálogos/contratos; lecturas paginadas sin efectos, cursores contextualizados y acciones idempotentes, sin rutas/shell arbitrarios (`meeting-notes`).

## 2. Runtime y audio nativo

- [ ] 2.1 Preparar runtime/modelos por versión/hash/licencia/capacidad, con rutas locales y comprobación de soporte; impedir descarga implícita, mantener CPU común y separar el perfil LLM (`meeting-notes`).
- [ ] 2.2 Implementar captura macOS ScreenCaptureKit/AVAudioEngine de micrófono y audio remoto, reloj/pistas, permisos e identidad del helper del producto; publicación durable de fragmentos administrados (`meeting-notes`).
- [ ] 2.3 Implementar captura Windows WASAPI y capacidades de loopback por proceso/equipo, nombres de fuente reales, dispositivos, reloj y cancelación; empaquetado por arquitectura (`meeting-notes`).
- [ ] 2.4 Implementar supervisor/host vivo, protocolo privado acotado, cancelación, una captura por host y cierre de dispositivos al pausar/perder propietario; detectar archivos parciales y recuperar sin reanudar captura (`meeting-notes`, `local-workspace`).

## 3. Transcripción, actores y conclusiones

- [ ] 3.1 Integrar streaming ASR/diarización con buffers externos, VAD/tratamiento de eco por referencia, offsets, etiquetas estables y publicación fenced de parciales/finales; conservar originales y representar huecos, solapamiento y capacidades reales (`meeting-notes`).
- [ ] 3.2 Implementar correcciones CAS, renombrar/agrupar/reasignar actores, vínculos People y sugerencias con evidencia/confirmación; conservar original y overlays al reprocesar (`meeting-notes`).
- [ ] 3.3 Implementar conclusiones estructuradas con tramos y notas manuales, perfil local/remoto explícito, fuentes/revisiones y resultado desactualizado; borradores de acciones con aceptación idempotente, sin herramientas de escritura del LLM (`meeting-notes`).

## 4. Interfaz y detección

- [ ] 4.1 Implementar bloque Meeting/documento v4 y `/meet` con guardias, referencia pendiente/vinculada y NodeView; conservar v1/v2/v3, colecciones mezcladas y recuperación JSON de incompatibles (`block-editor`).
- [ ] 4.2 Implementar Notas/Transcripción/Conclusiones/Participantes, fuente/tiempo/indicador, pausa/finalización, reproducción temporal y edición de actores; teclado, temas, virtualización/paginación y ancho estrecho (`desktop-shell`).
- [ ] 4.3 Integrar biblioteca bajo People y página desde Notes/Meetings con This Week/All/By People/Calendar, fecha civil, original/breadcrumb y vacíos reales; conservar las siete entradas laterales (`desktop-shell`, `meeting-notes`).
- [ ] 4.4 Implementar señales locales de posible llamada, popup sin robo de foco, deduplicación/silenciado y preferencias; inicio manual siempre disponible y cierre con captura activa protegido, sin autoarranque por abrir apps o por recuperar (`desktop-shell`).

## 5. Aceptación y cierre

- [ ] 5.1 Compilar núcleo/worker/desktop y contratos/catálogos; revisar límites, CAS y conservación frente a baseline independiente. No agregar ni ejecutar suites sin solicitud de prueba/verificación (`meeting-notes`, `block-editor`, `local-workspace`).
- [ ] 5.2 Inspeccionar flujo/captura real en Mac sobre copia, con fuentes acreditadas: auriculares/altavoces, permisos, pausa, pérdida de fuente, cierre, recuperación y backup busy/finalizado; registrar alcance y fallos sin activar una reunión ajena (`meeting-notes`, `desktop-shell`, `local-workspace`).
- [ ] 5.3 Completar aceptación equivalente en Windows real y registrar versión/arquitectura/capacidades. Una compilación o simulación en Mac no completa esta tarea (`meeting-notes`, `desktop-shell`).
- [ ] 5.4 Revisar calidad en español y recursos en hardware objetivo con muestras acreditadas: texto, nombres, hablantes/solapamiento, notas combinadas, decisiones/acciones y fuentes; contrastar candidatos y documentar errores/latencia sin trasladar benchmarks externos (`meeting-notes`).
- [ ] 5.5 Actualizar guías/ADR/plan y evidencia, validar OpenSpec y sincronizar sólo comportamiento implementado/aceptado antes de archivar; preservar requisitos y escenarios heredados (`meeting-notes`, `block-editor`, `desktop-shell`, `local-workspace`).
