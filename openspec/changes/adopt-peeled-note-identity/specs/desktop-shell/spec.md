# Spec Delta

## ADDED Requirements

### Requirement: Identidad local de nota adhesiva

El escritorio SHALL usar la «n» azul de nota adhesiva con esquina despegada en bienvenida, sidebar e ícono de la aplicación. El espacio liberado por la esquina MUST revelar la superficie inferior. La marca y el nombre de peso liviano SHALL funcionar en ambos temas, offline y sin cambiar navegación, foco o datos.

#### Scenario: Marca en ambos temas
- **WHEN** se muestra la bienvenida o un workspace en claro u oscuro
- **THEN** la misma silueta reemplaza `n.`, conserva el hueco exterior de la esquina y aparece junto al nombre legible sin agregar un control o requerir red

#### Scenario: Accesibilidad y tamaño compacto
- **WHEN** se usa un sidebar estrecho, contraste aumentado o movimiento reducido
- **THEN** el nombre y la silueta siguen visibles, la marca no ocupa el orden de teclado y no requiere animación o filtros de vidrio

### Requirement: Ícono nativo por capas en macOS

La aplicación macOS SHALL incluir el ícono por capas con variantes clara y oscura para que el sistema renderice Liquid Glass. El paquete temporal de desarrollo MUST incluir esos recursos y sus referencias. SHALL conservar un ícono de compatibilidad para entornos sin ese material, sin modificar la identidad de la app ni permisos.

#### Scenario: Paquete macOS y desarrollo
- **WHEN** se construye el paquete macOS o se prepara su paquete temporal de desarrollo
- **THEN** sus recursos incluyen el catálogo por capas y el ícono compatible, con referencias válidas y el identificador existente

#### Scenario: Compatibilidad y recursos locales
- **WHEN** se carga un entorno que usa el ícono convencional o el favicon
- **THEN** aparece la misma nota adhesiva desde recursos locales y no se descarga contenido ni se altera el workspace
