# Autenticación de proveedores

`noteka-auth` configura credenciales fuera del núcleo offline. Los tokens se guardan en el llavero del sistema (Keychain en macOS) y nunca se imprimen. Binario:

```sh
cargo build --release --offline --locked -p noteka-providers
export NOTEKA_AUTH="${CARGO_TARGET_DIR:-target}/release/noteka-auth"
```

## Sign in with ChatGPT

Estado local: sesión verificada con la cuenta personal del usuario; `gpt-6.1-sol`
respondió por CLI y por el proxy. Ver [verificación](verification-provider-auth.md).

```sh
"$NOTEKA_AUTH" openai login
```

Abre el navegador en el flujo oficial OIDC + PKCE (`auth.openai.com`), con callback loopback en `127.0.0.1` y scopes `openid profile email offline_access resource.invoke chatgpt.tokens.use.direct`. El primer registro usa `dynamic_agent_client` y el callback entrega el `client_id` emitido; no hay client secret. Al completar, la sesión queda en Keychain.

```sh
"$NOTEKA_AUTH" openai status
"$NOTEKA_AUTH" openai logout
```

Para usar ChatGPT desde el harness sin exponer el token, iniciá el proxy local:

```sh
"$NOTEKA_AUTH" serve chatgpt --port 11434
# En otra terminal, el harness puede apuntar a:
# http://127.0.0.1:11434/v1/chat/completions
```

El proxy transforma chat-completions a Responses y mantiene el protocolo JSON del harness. No hay fallback automático a API key ni a un modelo local.

## API keys

```sh
printf '%s' "$OPENAI_API_KEY" | "$NOTEKA_AUTH" key set openai
printf '%s' "$GEMINI_API_KEY" | "$NOTEKA_AUTH" key set gemini
"$NOTEKA_AUTH" key status openai
"$NOTEKA_AUTH" provider chat --provider openai --model <modelo> --prompt "Hola"
"$NOTEKA_AUTH" provider chat --provider gemini --model <modelo> --prompt "Hola"
```

También se aceptan `NOTEKA_OPENAI_API_KEY` y `NOTEKA_GEMINI_API_KEY` como fallback explícito. OpenAI usa `/v1/responses`; Gemini `generateContent`. Los modelos se eligen por CLI y no se fijan por nombre en el código.

## Google Drive

Estado local: **conectado y verificado** con la cuenta personal del usuario; la
configuración se creó en el proyecto `noteka-drive`. Ver
[verificación](verification-google-drive.md).

Para una instalación nueva:

1. En Google Cloud Console, habilitá **Google Drive API** y **Google Picker API**.
2. Configurá la pantalla de consentimiento en modo **Prueba** y agregá tu cuenta
   como usuario de prueba.
3. Agregá el permiso `https://www.googleapis.com/auth/drive.file`.
4. Creá un **OAuth client ID** de tipo **Desktop app**.
5. Guardá las credenciales fuera del repositorio y cargalas:

```sh
install -m 600 /dev/null "$HOME/.config/noteka/google-oauth.env"
# Editá el archivo con:
# export NOTEKA_GOOGLE_CLIENT_ID="....apps.googleusercontent.com"
# export NOTEKA_GOOGLE_CLIENT_SECRET="..."
. "$HOME/.config/noteka/google-oauth.env"
"$NOTEKA_AUTH" google login
"$NOTEKA_AUTH" google status
"$NOTEKA_AUTH" google files
"$NOTEKA_AUTH" google pick
"$NOTEKA_AUTH" google selected
"$NOTEKA_AUTH" google preview --id <picked-file-id>
```

El login abre el navegador con PKCE y callback loopback, y pide consentimiento
explícito. Scope mínimo: `drive.file` + identidad. `google files` lista hasta 20
metadatos; no descarga ni modifica archivos. Con `drive.file` el listado sólo
muestra archivos que la app creó o abrió, así que puede estar vacío.

`google pick` habilita **Google Picker API** y abre el selector oficial con una
autorización separada que pide sólo `drive.file` (`trigger_onepick=true`). El
usuario navega y elige hasta 20 archivos; Noteka guarda los IDs y
`google selected` devuelve sus metadatos. No se solicita `drive.readonly` ni
`drive.metadata.readonly`, no se enumeran todos los archivos y no se descarga ni
modifica contenido. Cancelar el Picker no borra la selección anterior.

`google preview` exporta un Google Doc como texto plano o un Google Sheet como
CSV de la primera hoja, con la sesión Picker y un límite de 2 MiB. No persiste el
contenido. La UI muestra esa vista previa de sólo lectura y permite abrir o
desvincular. La decisión está en
[ADR0032](decisions/0032-google-picker-drive-file.md).

En esta máquina el archivo local está en `~/.config/noteka/google-oauth.env`
(`0600`) y `~/.zshenv` lo carga si existe. No se versiona.

## Límites

- Login interactivo: requiere navegador y consentimiento del usuario.
- ChatGPT: el plan/cuenta debe habilitar los scopes; un ID token válido no garantiza inferencia si falta `chatgpt.tokens.use.direct`.
- Google: sin client ID/secret no hay login; no se inventan credenciales.
- El proxy sólo escucha `127.0.0.1`; no expone tokens al núcleo.
- No se versionan tokens, keys ni client secrets. No se activa nada al abrir/buscar.
