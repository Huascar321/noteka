# Design

## Componentes

`noteka-providers` aísla red, OAuth y credenciales del núcleo Rust determinista. `noteka-auth` expone login/status/logout, API keys, chat de prueba y un proxy loopback.

## ChatGPT

Se usa el flujo OSS de Sign in with ChatGPT: `auth.openai.com/api/accounts/authorize` con PKCE S256, `client_id=dynamic_agent_client` en el primer registro, `agent_name_hint=Noteka`, `ext_agent_host_id` estable, callback `http://127.0.0.1:<puerto>/auth/callback`, scopes `openid profile email offline_access resource.invoke chatgpt.tokens.use.direct` y resource `https://api.openai.com/v1`. El callback entrega el `client_id` emitido; el token se cambia en `/api/accounts/oauth/token` sin secret. El ID token se valida contra discovery/JWKS (firma, issuer, audience, exp, nonce). Se guarda access/refresh/id token en Keychain; nunca se imprimen.

Para reautorizar se usa el client_id emitido y `id_token_hint`; no se reenvía `agent_name_hint`/host en ese caso. Refresh usa el token endpoint.

## API keys y Gemini

`key set openai|gemini` lee stdin y guarda en Keychain; también se acepta `NOTEKA_OPENAI_API_KEY`/`NOTEKA_GEMINI_API_KEY` como fallback explícito. OpenAI usa `/v1/responses`; Gemini usa `generateContent`. Los modelos se pasan por CLI, no se hardcodea un modelo no verificado.

## Proxy

`noteka-auth serve chatgpt --port 11434` expone sólo loopback un endpoint compatible chat-completions, transforma mensajes a Responses y devuelve `choices[0].message.content`. El harness puede apuntar a ese loopback sin recibir tokens. No hay tool calling nativo en el proxy; el protocolo JSON del harness sigue siendo el contrato.

## Seguridad y límites

HTTPS obligatorio salvo loopback de prueba; timeouts 20–120s; respuestas acotadas; tokens en Keychain; no se registran prompts/tokens. El usuario debe autorizar el navegador. No se activa ningún proveedor al abrir o buscar. No hay fallback automático: si el proveedor falla, el error es visible.

## Verificación

Login ChatGPT real, `gpt-6.1-sol`, proxy y harness verificados. Selección explícita,
timeout, sesión/key ausente, proxy inválido y redacción de tokens están cubiertos
por pruebas. La prueba con API keys reales de OpenAI/Gemini y cuotas/capacidades
reales queda **diferida por decisión del usuario** (22/10/2026); las pruebas
locales no sustituyen esa verificación y no hay fallback automático.
