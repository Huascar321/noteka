# Verificación de Google Drive

Estado: **OAuth de escritorio y Google Picker verificados con la cuenta real** el
22/10/2026. OAuth, Picker, asociaciones, UI básica, apertura externa y preview
read-only de Docs/Sheets están verificados. Este documento cubre autenticación,
Picker, metadatos y preview.

## Configuración creada

- Proyecto de Google Cloud: `noteka-drive`, sin organización.
- Google Drive API y Google Picker API: habilitadas.
- Google Auth Platform: app externa en estado **Prueba**.
- Usuario de prueba: la cuenta personal configurada por el usuario.
- Permiso configurado: `https://www.googleapis.com/auth/drive.file`.
- Cliente OAuth: **App de escritorio** (`Noteka Desktop`).
- Client ID: `888744892379-jk44m14uofo43ssack3sn6sogn84u1ll.apps.googleusercontent.com`.
- El client secret no se versiona. Queda en
  `~/.config/noteka/google-oauth.env` con permisos `0600`; `.zshenv` lo carga
  sólo si existe.

## Evidencia

`google status` quedó conectado, con `drive.file` y `picked_count: 1` después de
la primera selección. La identidad, el email y los IDs concretos se omiten de
este documento para no versionar datos personales.

```sh
"$NOTEKA_AUTH" google files
```

Antes del Picker devolvía una lista vacía, comportamiento esperado con
`drive.file`: el scope sólo permite ver archivos creados o abiertos
explícitamente por la app.

```sh
"$NOTEKA_AUTH" google pick
```

- Se abrió Google Picker con una autorización separada que pidió **sólo**
  `drive.file`, `prompt=consent` y `trigger_onepick=true`.
- Se seleccionó un Google Sheet existente; el callback devolvió un
  `picked_file_ids` y `google pick` guardó la sesión Picker separada.
- `google selected` devolvió los metadatos acotados del archivo elegido.
- Después de la selección, `google files` también lo lista: la concesión
  `drive.file` queda asociada a la app, no a un token efímero.

```sh
"$NOTEKA_AUTH" google selected
```

```json
{"data":{"picked_file_ids":["<id>"],"files":[{"id":"<id>","mimeType":"application/vnd.google-apps.spreadsheet","name":"<archivo>"}]},"ok":true}
```

Prueba negativa: se abrió el Picker otra vez y se pulsó **Cancelar**. El comando
devolvió `access_denied`, no guardó una selección parcial y `google selected`
siguió devolviendo la selección anterior.

## Preview read-only

`google preview --id <picked-id>` se probó con el Google Sheet seleccionado:

- exportación `text/csv`, **12.674 bytes**, 11 líneas;
- contenido no impreso ni guardado en el workspace;
- `google preview --id <id-inexistente>` devolvió `not_found`;
- un servidor loopback que respondió 403 devolvió `permission_denied`;
- con el API base apuntando a un puerto cerrado devolvió `provider_http` por
  conexión rechazada, sin presentar contenido anterior.

También se creó un Google Doc sintético (`Noteka Preview Test Doc`) con la frase
«ornitorrinco azul» y se seleccionó con Picker. `google preview` devolvió
`text/plain`, 147 bytes y la frase esperada; el documento quedó en Drive y no se
modificó durante el preview. **El 23/10/2026 se borró explícitamente** con una
operación puntual autorizada por el usuario (`204 No Content`); `google files` ya
no lo lista y `google selected` devuelve `not_found` para su ID.

El CSV exportado para preview se mantuvo en memoria y no se persistió. Decisión y alternativas: [ADR0032](decisions/0032-google-picker-drive-file.md).

## Costos y seguridad

- No se agregó un método de pago ni se activó un servicio pago.
- El proyecto quedó vinculado a la cuenta de facturación existente de la cuenta;
  la consola mostró **$0.00** de costo y no se habilitaron APIs pagas.
- Drive API, Picker API, la creación del proyecto y el cliente OAuth no generaron
  cargos.
- Los tokens quedaron en Keychain. El client secret está fuera del repositorio,
  en un archivo local `0600`.
- La app permanece en modo Prueba y limitada al usuario de prueba; no se publicó.
- `drive.file` no permite enumerar todo el Drive ni solicitar scopes restringidos.

## Límites actuales

- Sólo autenticación, estado, logout, Picker, selección y metadatos acotados.
- Asociaciones, UI, preview Docs/Sheets y apertura externa verificados; el cambio
  queda listo para sincronizar/archivar.
- La eliminación del Doc sintético fue una operación puntual autorizada, no una
  capacidad de escritura/borrado del producto. No hay escritura de archivos ni de
  celdas en la API.
- No se probó deshabilitar Google Picker API después de habilitarla; el escenario
  de error queda cubierto por el error HTTP de Google, no por una prueba real.
