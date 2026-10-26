# Meeting Notes: diseño de la función

Fecha: 24/10/2026. **Propuesta, sin implementación de audio/ASR.** El usuario
eligió diseño y modelos primero. Cambio: `add-local-meeting-notes`.

## Experiencia que se propone

### 1. Iniciar

- Escribís **`/meet` dentro del editor de Noteka** o elegís New Meeting Note.
  Se prepara un componente, con título, notas y asistentes opcionales.
- Si habilitás detección, con Noteka ejecutándose aparece un popup discreto
  **“Posible reunión en Google Meet. ¿Querés grabarla?”**, con Iniciar/No ahora.
  No roba el foco. Se evita repetirlo por la misma llamada y podés silenciar avisos.
- El inicio muestra la fuente concreta: micrófono y audio de aplicación/equipo.
  Aceptar Iniciar enciende la captura y los modelos ya preparados. No hay otro
  paso de consentimiento por participante dentro de Noteka.
- La primera preparación resuelve permisos del sistema y descarga acreditada de
  modelos. macOS/Windows pueden exigir sus propios permisos; Noteka no los omite.
  Si faltan modelos o permisos, lo informa antes de aparentar que está transcribiendo.

El detector combina señales de audio y aplicación/ventana; abrir Zoom por sí
solo no demuestra una reunión. Meet en navegador y Zoom/Teams nativos son los
primeros escenarios a validar. Otras aplicaciones conservan inicio manual.
Cerrar Noteka desactiva el detector; minimizar la ventana permite seguir usándolo.

### 2. Durante la reunión

Un indicador visible muestra estado, tiempo y fuentes. **Pausar** detiene captura
de ambas pistas; **Reanudar** continúa con un hueco temporal explícito;
**Finalizar** conserva audio/tramos y comienza la finalización del texto.
El micrófono muteado o el silencio no terminan automáticamente la sesión.

Podés escribir notas manuales mientras aparece la transcripción provisional.
El audio remoto se obtiene directamente del sistema, también con auriculares;
no se intenta volver a escucharlo por el micrófono. Se conservan pistas separadas
para distinguir fuentes y reducir atribuciones/eco incorrectos.
El preprocesado usa el audio remoto como referencia para tratar el eco; no
sustituye ni modifica el audio original conservado.

La fuente de navegador puede abarcar más de una pestaña si el SO filtra por
aplicación; la UI debe decirlo. No se cambia a “todo el audio del equipo” ante un
fallo sin que el usuario elija esa fuente. Cambiar de pestaña de Noteka no cancela
la grabación ni guarda otros borradores. Cerrar resuelve la captura activa.

### 3. Después de finalizar

| Pestaña | Contenido |
|---|---|
| **Notas** | Lo que escribiste, agenda y contexto de la reunión; editable sin IA. |
| **Transcripción** | Tramos por hablante con tiempos, búsqueda, texto corregible y reproducción desde el fragmento. |
| **Conclusiones** | Resumen, decisiones, dudas abiertas y acciones propuestas; cada elemento enlaza a sus fuentes. |
| **Participantes** | Hablantes detectados, nombres/roles y vínculo opcional a People; agrupar o reasignar tramos. |

Las conclusiones usan **transcripción más notas manuales**. Se generan al
finalizar si hay un perfil LLM elegido. Si falta o falla, la reunión conserva
audio, notas y transcripción, con una acción explícita para configurar/reintentar.
Una cita válida permite revisar el origen; no garantiza que el resumen sea correcto.

Las acciones son borradores. Podés corregir responsable/plazo y elegir cuáles
convertir en Tasks. Repetir una confirmación no duplica tareas. No se envían
correos o mensajes por inferencia del modelo.

## Cómo se resuelven los actores

El sistema detecta cambios/reapariciones de voz y mantiene etiquetas estables
**Hablante 1, Hablante 2…**. Separar voces no identifica automáticamente a una
persona real. Los asistentes que elijas y una presentación explícita en el texto
pueden ayudar a proponer un nombre con evidencia; cualquier propuesta queda
marcada como sugerida hasta confirmarla.

Después podés renombrar, vincular a People, unir etiquetas o reasignar un tramo.
El original del modelo y tus correcciones se conservan por separado. Un nuevo
procesamiento no pisa tu trabajo. No se crean personas o perfiles biométricos por
haber escuchado una voz. El perfil candidato admite ocho voces automáticas;
una reunión mayor requiere valorar otro diarizador o corregir su atribución.

## Biblioteca integrada con People y Notes

Debajo de la galería People aparecerá **Meeting Notes**, con New Meeting Note y
las vistas **This Week / All / By People / Calendar**. Desde Notes/Meetings se
abrirá su página propia, con breadcrumb Home → Notes → Meeting Notes → reunión.
La misma sesión conserva su ID aunque se abra desde una persona, nota o búsqueda.

This Week usa lunes–domingo del workspace; Calendar muestra la fecha de reunión,
no la última edición. By People usa participantes vinculados explícitamente y
un grupo Sin asignar. Las reuniones vacías/manuales siguen siendo editables.
No se añade otra entrada primaria al lateral.

## Motor propuesto

**Nemotron 3.5 ASR Streaming 0.6B + Nemotron 3 Diarization**, mediante
**NeMo-Speech.cpp**, es el candidato inicial. ASR, separación de voces y resumen
son etapas independientes: se pueden corregir o sustituir sin perder originales.
La selección, alternativas y evidencia están en [modelos](meeting-notes-models.md).

El audio/ASR quedan locales. El LLM de conclusiones se configura por separado;
se prioriza un perfil local y el experimento LoRA existente no es obligatorio.
Usar un proveedor remoto exige elegirlo para Meeting Notes y saber qué contenido
se enviará. No se hereda silenciosamente una preferencia de otro uso de IA.

“Universal” significa el mismo dominio/protocolo y un camino CPU común en Mac y
Windows; la captura y aceleración tienen adaptadores por sistema. No significa
que un modelo pueda acceder al audio de cualquier SO sin permisos ni integración.

## Conservación y recuperación

- SQLite conserva sesiones, tramos, actores, correcciones y generaciones. El audio
  original se guarda como adjuntos administrados con hashes; nunca en Git.
- Notas manuales, salida ASR original, correcciones y conclusiones mantienen su
  identidad/revisión. Cambiar el texto señala qué conclusiones necesitan revisión.
- Tras una caída se identifica la sesión interrumpida sin modificarla al leer.
  **Recuperar** verifica y conserva lo guardado mediante una acción explícita;
  no vuelve a abrir micrófonos. Se puede reprocesar el audio conservado.
- Un backup durante captura responde ocupado. Finalizar publica el último audio
  y permite un respaldo consistente que incluye los originales.
- Retener/borrar audio será una decisión explícita. Borrarlo afecta reproducción
  y reprocesamiento; no debe borrar silenciosamente las notas o la transcripción.

Límites iniciales propuestos: una captura por equipo, ocho horas por sesión,
100.000 tramos, 32 MiB de texto y fragmentos de audio dentro de los 64 MiB
existentes. Dos pistas PCM16/16 kHz ocupan aproximadamente **230 MB por hora**,
antes de cualquier compresión futura. La pantalla de preparación muestra espacio
y tamaños reales. Alcanzar un límite conserva lo confirmado y señala el cierre.

## Antes de presentarlo como operativo

Se debe comprobar captura real con auriculares, permisos, cambios de dispositivo,
pausa/cierre y recuperación en **Mac y Windows por separado**. Además, revisar
español, nombres propios, voces superpuestas y atribución de acciones con muestras
acreditadas. No hay todavía mediciones propias de WER, DER, latencia o RAM.

El [diseño técnico](../openspec/changes/add-local-meeting-notes/design.md), las
[tareas](../openspec/changes/add-local-meeting-notes/tasks.md) y
[ADR0046 propuesto](decisions/0046-local-meeting-notes-runtime.md) fijan el trabajo.

## Referencias de producto

Se consultó la [ayuda actual de AI Meeting Notes](https://www.notion.com/help/ai-meeting-notes)
y la transcripción automática en inglés de [Introducing AI Meeting Notes](https://www.youtube.com/watch?v=f4ZYEZO3Sq0).
La referencia aporta el componente, inicio manual, notas durante la llamada,
resumen y acciones posteriores. No se atribuye esa capacidad a Noteka por observarla
en Notion ni se promete paridad completa.
