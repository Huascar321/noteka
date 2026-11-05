# Verificación de clima en Home

Cambio `add-home-weather`, 26/10/2026. Build de producción y diagnósticos Svelte sin errores; prueba de cliente y almacenamiento con respuestas simuladas, y recorrido Chromium aislado de selección, ocho fechas, recuperación offline y desactivación. `openspec validate --strict` verifica documentos. El host Tauri recompiló y arrancó con CSP actualizada; no se probó la consulta real ni el flujo de red en la ventana nativa, y el servicio externo puede fallar o cambiar sus condiciones. La búsqueda simulada no utilizó datos personales ni habilitó una ciudad real.

La ampliación de ubicación actual se compiló en macOS y se probó con IPC y red simulados en Chromium: activación explícita, reapertura, ciudad fija, permiso denegado, cancelación y respuesta tardía. La ubicación física, el diálogo de macOS y el servicio de Apple no se ejercitaron; requieren aceptación nativa. La geocodificación inversa usa CLGeocoder, deprecado desde macOS 26, pendiente de sustitución por MapKit.

Actualización posterior: Set location invoca directamente la ubicación; la ciudad fija anterior se ignora. Prueba Chromium con IPC y respuestas simuladas cubrió ausencia de consultas previas, migración, permiso denegado, reintento, ocho fechas y desactivación. La ubicación y el permiso reales en macOS permanecen sin verificar.

Diagnóstico posterior: macOS registró la solicitud de autorización con identidad de ejecutable de desarrollo, pero el usuario indicó que no apareció el aviso. Se separaron plazos de autorización y ubicación, y el nombre ahora tiene fallback por coordenadas. Compilación y estados de plataforma simulados aprobados; todavía no hay evidencia de que el permiso real aparezca ni de pronóstico real nativo. Windows y Linux aún no implementan geolocalización y la interfaz lo indica.

Tras recompilar con plazos separados, el usuario volvió a probar y confirmó que el aviso de macOS sigue sin aparecer. El timeout no está resuelto; la identidad de ejecutable no empaquetado es una hipótesis sin confirmar. No se afirma aceptación nativa.

Corrección verificada el 26/10: una copia empaquetada identificada como app.noteka.desktop presentó el aviso; el usuario concedió permiso y se observó el pronóstico real con ocho fechas. El arranque debug ahora empaqueta el host y sidecars y los abre mediante Launch Services; el host así iniciado volvió a obtener ubicación y actualizar el pronóstico real. Se verificó una recarga de desarrollo, con terminación del proceso anterior, eliminación del paquete temporal y presencia de los cinco sidecars macOS. Los datos de prueba del workspace estaban aislados; no se registran coordenadas ni nombres de localidad. No valida geolocalización Windows/Linux ni distribución firmada.

Animaciones meteorológicas: se verificaron los mapeos WMO, condición actual día/noche y lectura de caché anterior sin is_day con pruebas unitarias. Una galería Chromium sintética permitió revisar las familias de iconos; se comprobó pausa y movimiento reducido mediante estilos computados. Build de frontend sin errores ni advertencias Svelte; la revisión visual no equivale a aceptación nativa en todas las plataformas.

Diseño de franja: revisión Chromium con datos sintéticos en ancho amplio y 480 px; ocho días presentes, desplazamiento horizontal con teclado, atribución y controles conservados, fondo integrado en tema claro y oscuro. Sin revisión visual nativa adicional.

Controles discretos: prueba Chromium con puntero, foco por teclado, emulación táctil y movimiento reducido; el pie aparece sin mover el contenido y permanece accesible. No se realizó una revisión nativa adicional.
