# Design

## Context

Home contiene una franja pendiente. La API pública de Open-Meteo ofrece pronósticos sin clave para uso no comercial.

## Goals / Non-Goals

Ubicación actual optativa, datos actuales y ocho fechas, privacidad explícita, caché y acceso offline. Sin selección manual. Sin secretos ni cambios de SQLite.

## Decisions

- Set location solicita ubicación directamente; el aviso visible identifica destinatarios y datos enviados. Desactivar borra ubicación y caché.
- Preferencias y caché de clima versionadas en localStorage, separadas de documentos SQLite. Datos remotos se validan antes de mostrarse; errores no bloquean Home.
- HTTPS exclusivamente al dominio de pronóstico mediante CSP. Pronóstico con zona de la ciudad, ocho días, máximo/mínimo y códigos WMO. Caché con hora visible; actualización al iniciar y cada hora.
- En macOS Core Location obtiene ubicación con permiso explícito; una geocodificación inversa de Apple puede enviar coordenadas a Apple para el nombre. Se envían coordenadas a Open-Meteo, se redetecta al abrir y cada hora. La autorización, la obtención de coordenadas y el nombre tienen plazos independientes; si el nombre no llega se usan coordenadas sin localidad. Si se rechaza o falla, conserva caché y ofrece reintento. En otras plataformas el texto identifica que la detección automática aún no está disponible y no ofrece un control que falle.
- En macOS de desarrollo, el host debug se empaqueta temporalmente con su Info.plist y sidecars, se firma ad hoc y se abre por Launch Services para que el sistema identifique la aplicación. Un supervisor termina la copia al cerrar o recargar y elimina el paquete temporal; la distribución y otros sistemas no cambian.
- La franja del clima sigue la referencia: ubicación, icono y condición actual junto a columnas diarias; sin tarjeta de fondo, adaptable a ancho estrecho con desplazamiento de días. Controles y atribución aparecen con transición al pasar el cursor o al recibir foco; en dispositivos sin hover están siempre visibles.
- Iconos meteorológicos locales de trazo mínimo con animación por código WMO; la condición actual usa is_day, las fechas diarias son resúmenes diurnos. Respetan movimiento reducido y se detienen fuera de Home visible o en segundo plano. Se conserva atribución como enlace discreto Open-Meteo. Cachés anteriores sin is_day siguen siendo legibles.

## Risks / Trade-offs

El proveedor puede fallar y el acceso gratuito alojado es no comercial; se comunica la antigüedad del caché. Las coordenadas se transmiten al proveedor únicamente tras interacción explícita.

## Migration Plan

Los datos previos de ciudad fija se ignoran sin borrarlos ni activar ubicación; al elegir Set location se reemplazan tras una consulta exitosa. Borrar la preferencia desactiva y elimina caché; formatos inválidos se ignoran.
