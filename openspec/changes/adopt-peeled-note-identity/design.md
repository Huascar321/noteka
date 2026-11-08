# Design

## Context

Ver `proposal.md` para motivación. La UI usa `n.` textual y `public/noteka.svg` contiene el ícono verde antiguo. Tauri sólo referencia un PNG de 256 px. macOS debug crea un `.app` temporal sin recursos de ícono. La máquina dispone de Xcode 27.2, `ictool` y `actool`; Tauri 2.12 admite catálogos `Assets.car`.

## Goals / Non-Goals

**Goals:** mantener una geometría vectorial única, renderizar el material nativo, conservar compatibilidad y verificar el empaquetado sin abrir workspaces personales.

**Non-Goals:** publicación, notarización, instalación global, modificar proveedores, reglas de dominio, modelos o datos. La apariencia dinámica depende de macOS; una captura estática no acredita todos los fondos o versiones.

## Decisions

1. Geometría y colores de marca en JSON local compartido por un componente SVG y el generador. Evita divergencias entre UI, favicon y capas. La esquina liberada está fuera del contorno; la solapa clara ocupa una capa independiente. Se descarta recortar la lámina generada: es una referencia conceptual con sombras y texto, no un recurso de producción.
2. Sidebar/bienvenida usan SVG pequeño, estático, sin filtros ni relieve grande. Azul de marca fijo y nombre de peso 450; el acento personal conserva sus controles e iconografía. Se reutilizan las superficies y tamaños existentes.
3. Documento `Noteka.icon` con fondo neutro, cuerpo azul y solapa clara; efectos definidos por Icon Composer. `ictool` exporta el fallback desde ese documento y `actool` compila `Assets.car`. El render nativo reemplaza la simulación inicial como evidencia de implementación.
4. Recursos compilados versionados con huellas de fuentes/salidas y un comando explícito de regeneración. Los builds ordinarios comprueban integridad sin exigir Icon Composer en otras plataformas. Regenerar material Apple requiere Xcode compatible, sin descargas automáticas.
5. Configuración macOS referencia el catálogo. El paquete debug incluye el catálogo y `.icns` mediante recursos embebidos, y conserva Launch Services, firma local y permisos actuales. No se modifica Core ni SQLite.

## Risks / Trade-offs

- El pequeño pliegue puede perder definición al reducir → revisar 16/24/32 px y capturas del sidebar.
- Los efectos cambian entre versiones del sistema → usar el render nativo y conservar `.icns`/PNG/ICO de compatibilidad; registrar versión usada.
- Caché de íconos de macOS → el recurso nuevo entra al reconstruir/reabrir; no reiniciar Dock ni borrar cachés globales.
- Recursos compilados desactualizados → comprobación de huellas vinculada al build y comandos de regeneración documentados.

## Migration Plan

Sólo recursos de aplicación; no hay migración de datos. Revertir fuentes, referencias y recursos de este cambio restaura la identidad previa.
