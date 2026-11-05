# Tasks

## 1. Coordinación

- [x] 1.1 Implementar el pool de lecturas por workspace con cancelación en cola y cupos físicos; verificar concurrencia, cancelación y limpieza con pruebas deterministas.
- [x] 1.2 Agrupar ráfagas de Home por identidad y página; verificar una consulta inicial y como máximo una posterior con promesas retenidas.
- [x] 1.3 Coordinar los demás refrescos por panel, encolar mientras hay guardado/bloqueo e invalidar lecturas al navegar; verificar foco, activación y resultados tardíos.

## 2. Presentación y seguridad

- [x] 2.1 Conservar resultados de la misma consulta, identificar fallo y reinicio de páginas y deshabilitar cursores anteriores; probar cambio de vista, fecha, error y reintento.
- [x] 2.2 Proteger el tablero con resultados vigentes y preservar borradores y conflictos; verificar movimientos, cambios externos y edición pendiente.

## 3. Verificación

- [x] 3.1 Ejecutar pruebas enfocadas y build, incluyendo navegador headless con CLI real y mediciones de 30 ráfagas sobre 1.000 y 8.000 entidades.
- [x] 3.2 Comprobar foco físico macOS en ventana Tauri y workspace sintético autorizados, y documentar resultados y límites.
- [x] 3.3 Comparar mediciones antes/después, documentar resultados y fallos heredados, validar OpenSpec y registrar compatibilidad.
