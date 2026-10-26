# Proposal

## Why

El harness local ya funciona, pero el usuario quiere usar su plan de ChatGPT, una API key de OpenAI y Gemini. Falta autenticación explícita, almacenamiento seguro de credenciales y un adaptador que no convierta un fallo local en una llamada remota silenciosa.

## What Changes

- Nueva crate `noteka-providers`/CLI `noteka-auth` con:
  - Sign in with ChatGPT por OIDC + PKCE, callback loopback y registro dinámico (`dynamic_agent_client`), sin client secret.
  - API keys de OpenAI y Gemini en Keychain (o variables de entorno explícitas).
  - Proxy loopback `/v1/chat/completions` para que el harness use ChatGPT sin exponer tokens al núcleo.
- Sesiones con refresh, status/logout y errores estructurados; tokens nunca se imprimen.
- Selección explícita de proveedor; sin fallback automático entre ChatGPT, API key y modelos locales.
- Límites de tamaño/timeout, scopes mínimos y ninguna activación al abrir un workspace.

## Capabilities

### New Capabilities
- `cloud-providers`: autenticación, credenciales, perfiles y transporte de proveedores cloud.

### Modified Capabilities
- Ninguna en este recorte; la integración del harness se completará tras la autenticación real.

## Impact

- Nueva crate/CLI fuera del núcleo offline: `ureq`, `jsonwebtoken`, `keyring`.
- No cambia SQLite/API/MCP ni el núcleo; no se versionan tokens.
- Requiere intervención del usuario para completar el login ChatGPT y, si corresponde, API keys/cuotas.
