# Design

## Context

El scroll actual es nativo. `scroll-behavior` no suaviza entrada manual y WheelEvent no ofrece una identificación inequívoca de mouse versus trackpad.

## Goals / Non-Goals

**Goals:** atenuar pasos discretos sin alterar distancia, límites o datos, con salida al comportamiento nativo.

**Non-Goals:** identificar infaliblemente hardware, reemplazar gestos precisos, zoom o barras nativas, ni certificar percepción en todos los dispositivos.

## Decisions

Un controlador local utiliza desplazamiento suave del navegador para eventos verticales con señal discreta conservadora (unidades de línea o múltiplos de rueda heredados); eventos sin señal, horizontales, con modificadores, controles especiales y límites se delegan al motor. Se acumulan pasos de una misma dirección y se interrumpe ante acciones nuevas. La preferencia local predeterminada activada se puede desactivar; movimiento reducido usa desplazamiento nativo. El listener se instala sólo mientras la preferencia esté activa y se limpia al desmontar. Las reglas de anidación y propagación permanecen del navegador en límites.

## Risks / Trade-offs

- [La señal de WheelEvent es heurística y el listener cancelable puede afectar el compositor] → detección conservadora, controles excluidos, opción de desactivación y pruebas; la experiencia física requiere aceptación del usuario.
- [Plataformas con señales diferentes] → sin señal inequívoca se conserva scroll nativo.

## Migration Plan

Clave local nueva; sin cambios del workspace. Valor ausente activa la función, valor inválido usa predeterminado.
