# Proposal

## Why

El usuario necesita ajustar y recuperar el ancho del menú izquierdo y Side peek, con transiciones consistentes y espacio suficiente para el contenido. Pidió límites explícitos de 35% y 65%, exclusión del menú con un peek mayor al 35% y otro ajuste pequeño del indicador.

## What Changes

- Arrastre horizontal y teclado en los bordes de ambos paneles, con memoria local de ancho en píxeles y límites recalculados sobre el viewport de Noteka.
- Menú izquierdo máximo 35%; Side peek máximo 65%. Cuando Side peek supera 35%, abrirlo cierra el menú; abrir el menú oculta ese peek conservando su instancia y borrador.
- Side peek se expande/contrae en 260ms, como el lateral, con movimiento reducido y sin animación durante el arrastre.
- Marcas del indicador ligeramente más cortas y juntas, conservando su navegación.

## Capabilities

### New Capabilities

Ninguna.

### Modified Capabilities

- `desktop-shell`: dimensiones locales, exclusión de paneles y transición de Side peek con buffers conservados.

## Impact

Shell, componente de borde ajustable, preferencias del cliente y CSS. Se conserva el árbol de panes de sesión y su protección. No cambia Rust, SQLite, contratos públicos, dependencias, datos personales ni servicios externos. Las preferencias son globales del dispositivo; la sesión no se persiste. Center/Full/New tab conservan sus modos y límites anteriores.
