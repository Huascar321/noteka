# Tasks

## 1. ChatGPT

- [x] 1.1 Implementar PKCE, callback loopback, autorización y exchange sin secret.
- [x] 1.2 Validar ID token con discovery/JWKS y guardar sesión en Keychain.
- [x] 1.3 Implementar status/logout/refresh y proxy chat-completions.
- [x] 1.4 Completar login real del usuario y verificar cuenta/scopes/refresh inicial.

## 2. API keys

- [x] 2.1 Guardar/consultar/borrar keys en Keychain con fallback env.
- [x] 2.2 Implementar chat OpenAI Responses y Gemini generateContent.
- [ ] 2.3 Probar con cuentas/keys autorizadas y errores de cuota/capacidad. **Diferida 22/10/2026 por decisión del usuario: no hay keys reales configuradas; no bloquea el archivo. Cobertura local: key/sesión ausente, timeout, selección explícita y errores estructurados.**

## 3. Integración y verificación

- [x] 3.1 Conectar el proxy ChatGPT al harness y probar QA/citas (citation_required esperado).
- [x] 3.2 Probar selección explícita, ausencia de fallback, timeout y tokens no expuestos.
- [x] 3.3 Documentar, actualizar plan y archivar; 2.3 queda diferida y documentada.
