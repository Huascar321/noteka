# Verificación de autenticación de proveedores

Estado: **ChatGPT y Google Drive verificados con la cuenta del usuario** el
22/10/2026. Las API keys de OpenAI/Gemini siguen pendientes.

## ChatGPT

- Login OIDC+PKCE completado desde el navegador del usuario; callback loopback.
- Cuenta personal del usuario; subject validado.
- Scopes concedidos: `chatgpt.tokens.use.direct`, `email`, `offline_access`,
  `openid`, `profile`, `resource.invoke`; `chatgpt_plan=true`.
- Tokens guardados en Keychain; `openai status` no los imprime.
- `provider chat --provider chatgpt --model gpt-6.1-sol --prompt "Responde exactamente: conectado"` devolvió `conectado`.
- Proxy loopback `serve chatgpt --port 11434` respondió chat-completions.
- Harness real con el proxy: `agent ask` llegó al modelo, obtuvo un `final`
  JSON y falló correctamente con `citation_required` al no incluir cita ni
  `insufficient_evidence`. Esto valida el transporte y el contrato de citas, no
  la calidad de las respuestas.

## Google Drive

- Proyecto `noteka-drive`; Google Drive API y Google Picker API habilitadas.
- Cliente OAuth Desktop en modo Prueba, usuario de prueba y scope `drive.file`.
- `google status` conectado; antes del Picker `google files` devolvía una lista
  vacía, esperada con `drive.file`.
- `google pick` seleccionó un archivo existente con Picker, guardó una sesión
  separada y `google selected` devolvió sus metadatos; cancelar no borró la
  selección anterior.
- Evidencia y límites en [verificación Google Drive](verification-google-drive.md).

## Selección, timeout y tokens

- `provider chat` exige `--provider` explícito; no hay fallback entre ChatGPT,
  OpenAI, Gemini y modelos locales.
- `noteka-providers`: 18 pruebas, incluidas:
  - sesión ChatGPT ausente → `auth_required`;
  - API key ausente → `auth_required` antes de enviar contenido;
  - proveedor de key desconocido → `validation_error`;
  - proxy: ruta y modelo inválidos sin sesión, sin exponer credenciales;
  - `status` de ChatGPT no incluye access/refresh/id tokens;
  - timeout de transporte visible como `provider_http`.
- No se probaron cuotas/capacidades reales de OpenAI/Gemini porque no hay keys
  autorizadas configuradas.

## Cierre 6a

`add-cloud-model-providers` quedó **9/10 y archivado** como
`2026-10-22-add-cloud-model-providers`. La tarea 2.3 (keys reales de OpenAI/Gemini
y cuotas/capacidades) queda **diferida por decisión del usuario** y documentada
como limitación; no bloquea el archivo. Las pruebas locales cubren selección
explícita, ausencia de fallback, timeout, errores estructurados y no exposición
de tokens.

## Pendientes

- API keys OpenAI/Gemini reales y cuotas/capacidades: diferidas por decisión del
  usuario; no bloquean el archivo.
- El refresh real de ChatGPT tras expiración no se forzó; el flujo está
  implementado y los errores de sesión/key ausente están probados.
- 6b está archivado con asociaciones, UI y previews Docs/Sheets.

## Seguridad

No se imprimieron tokens. El proxy sólo escucha loopback y no los expone al
núcleo. No se envió contenido a proveedores sin una acción explícita del usuario;
la prueba de chat fue solicitada por el usuario y usó su cuenta.
