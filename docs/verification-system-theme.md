# Verificación: tema del sistema

El 27/10/2026, `npm run build` pasó con cero errores y advertencias de Svelte (Vite conserva el aviso de chunk >500 kB). La prueba focalizada de preferencias pasó con valores anteriores, System, resolución y error de almacenamiento. `node scripts/headless-system-theme-smoke.mjs` pasó con UI compilada, workspace sintético y apariencia emulada: orden de opciones, seguimiento claro/oscuro, paleta, borrador pendiente, opción fija, error de almacenamiento y reapertura de System.

La primera ampliación de la prueba falló por selectores del panel lateral y se corrigió; una ejecución posterior quedó interrumpida al probar reapertura con un diálogo de borrador, reemplazada por una página independiente. No se cambió la configuración real del sistema. La aceptación nativa de cambios de apariencia en macOS, Windows y Linux queda pendiente; el navegador emulado no prueba sus WebViews.

La suite completa `tests/home-reads.mjs` tuvo una falla previa ajena en el texto esperado de una respuesta incompatible: la prueba espera español y el código devuelve inglés. La prueba focalizada de tema sí pasó.
