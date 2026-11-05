# Spec Delta

## ADDED Requirements

### Requirement: Acceso lateral Search con cápsula de campo

El acceso Search del sidebar SHALL presentar una cápsula neutra con borde fino, tinte gris translúcido y reflejos interiores discretos en claro y oscuro. MUST conservar icono, etiqueta, atajo, altura y apertura del buscador mediante clic o teclado, con foco visible. Su material MUST seguir neutro al cambiar el color primario y ofrecer fondo sólido con transparencia reducida o contraste aumentado.

#### Scenario: Apariencia y apertura
- **WHEN** se muestra el sidebar y se activa Search con clic, teclado o el atajo de búsqueda
- **THEN** la cápsula conserva su tamaño y abre el buscador existente; Escape permite volver sin modificar contenido

#### Scenario: Temas y estados
- **WHEN** se cambia entre claro y oscuro, se modifica el acento o se pasa el cursor o enfoca Search
- **THEN** el material sigue neutro, el texto de la etiqueta mantiene contraste mínimo de 4,5:1 sobre el fondo compuesto y el foco permanece visible

#### Scenario: Vista estrecha y accesibilidad
- **WHEN** el sidebar se estrecha o se solicita menor transparencia, más contraste o menos movimiento
- **THEN** la cápsula permanece contenida y alcanzable; las preferencias de transparencia/contraste restablecen una superficie sólida sin reflejos y movimiento reducido elimina sus transiciones
