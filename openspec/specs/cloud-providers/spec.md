# cloud-providers Specification

## Purpose
Permitir que Noteka use ChatGPT y API keys de OpenAI/Gemini de forma explícita, segura y sin fallback silencioso, manteniendo el núcleo offline sin credenciales.

## Requirements

### Requirement: Autenticación ChatGPT explícita

El sistema SHALL ofrecer login de ChatGPT por OIDC + PKCE con callback loopback, registro dinámico y scopes mínimos de identidad/plan. MUST validar estado, nonce, firma, issuer, audience y expiración; MUST guardar tokens en el almacén seguro y MUST NOT imprimirlos.

El login MUST ser una acción explícita del usuario; abrir/buscar/consultar no SHALL iniciar autenticación ni enviar contenido. Logout SHALL borrar la sesión. El refresh MUST fallar de forma visible si no hay token válido.

#### Scenario: Login y refresh
- **WHEN** el usuario completa el navegador y el callback entrega un código válido
- **THEN** se guarda una sesión con identidad/scopes y el refresh renueva el acceso sin imprimir tokens

#### Scenario: Callback inválido o token vencido
- **WHEN** el estado/nonce/firma/audience no coinciden o el refresh falla
- **THEN** no se guarda la sesión y se informa un error de autenticación

### Requirement: API keys y proveedores explícitos

El sistema SHALL permitir configurar API keys de OpenAI y Gemini en el almacén seguro, con fallback explícito por variable de entorno, y elegir el proveedor por solicitud. MUST NOT cambiar automáticamente de ChatGPT a API key, de local a remoto ni de un modelo a otro.

Cada proveedor MUST validar modelo, endpoint HTTPS, timeout y tamaño de respuesta. Los errores de cuota, autenticación, capacidad o red MUST ser visibles y no SHALL producir una escritura parcial.

#### Scenario: Proveedor elegido
- **WHEN** se solicita chat con `chatgpt`, `openai` o `gemini`
- **THEN** se usa sólo ese transporte y se devuelve texto/error estructurado

#### Scenario: Key o modelo ausente
- **WHEN** falta la key, el modelo o el endpoint es inválido
- **THEN** la solicitud falla antes de enviar contenido y no usa otro proveedor

### Requirement: Proxy local sin exponer credenciales

El proxy ChatGPT SHALL escuchar sólo loopback y aceptar chat-completions acotado, transformándolo a Responses. MUST NOT exponer access/refresh tokens al cliente ni aceptar conexiones no loopback. El harness MAY usarlo como endpoint local, conservando su protocolo JSON y sus límites.

#### Scenario: Harness usa el proxy
- **WHEN** el harness apunta a `127.0.0.1` y el proxy tiene una sesión válida
- **THEN** recibe texto chat-completions sin ver tokens y conserva sus citas/límites

#### Scenario: Proxy sin sesión o request inválido
- **WHEN** falta la sesión, el body excede el límite o la ruta no es chat-completions
- **THEN** responde error local sin llamar al proveedor ni filtrar credenciales
