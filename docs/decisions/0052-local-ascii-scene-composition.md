# ADR 0052: composición local de tres variantes ASCII

Fecha: 26/10/2026. Estado: adoptada localmente.

## Context

El usuario pidió koi de una referencia pública y medusas de un ZIP, conservando
los tiburones. El renderer original tiene geometría y pausas ya verificadas;
reescribirlo para toda la fauna amplificaría el riesgo de regresión.

## Decision

Mantener AsciiOceanRenderer intacto y componerlo con dos renderizadores locales
de criaturas en AsciiCoverRenderer. Una semilla efímera determina un orden de tres
escenas; cada una ocupa 42 segundos activos y funde 1,4s con la siguiente. Las
capas y atlas se reutilizan, el frame actual se repinta sin esperar RAF al resize
y la alternancia usa OceanClock, sin timers o preferencias paralelas.

Koi y medusas usan siluetas procedurales propias rasterizadas a una grilla de
caracteres preparados, con señales separadas de familia/intensidad/opacidad.
Las referencias sirven para anatomía y apariencia; sus interfaces, fuentes,
instrucciones y servicios no se ejecutan ni se distribuyen.

## Alternatives

- Importar ambos proyectos: suma dependencias/interfaces y restricciones ajenas
  para un fondo decorativo, además de duplicar relojes y pausas.
- Sustituir el renderer de tiburones: arriesga las siluetas/nado que el usuario
  pidió conservar.
- Sortear por frame: provoca ruido y no deja tiempo para reconocer cada variante.

## Consequences

Hay hasta dos capas activas durante el fundido. El gasto se mide en navegador;
las cifras no acreditan refresco físico de 120Hz. Las sesiones no persisten su
semilla y la preferencia Photo/ASCII conserva su formato anterior. SQLite, Rust,
contratos y documentos permanecen fuera de este cambio.
