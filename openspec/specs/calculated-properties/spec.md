# calculated-properties Specification

## Purpose

Calcular propiedades derivadas deterministas y tipadas sobre originales compartidos, con errores observables y dinero exacto sin depender de modelos.

## Requirements

### Requirement: AST tipado y seguro

El núcleo SHALL validar y evaluar AST JSON acotados de literales, propiedades, aritmética, comparaciones, booleanos y condicionales. MUST rechazar sintaxis, tipos, referencias o ciclos inválidos sin ejecutar código arbitrario ni modificar la definición vigente.

#### Scenario: Expresión válida y tipos
- **WHEN** se define una fórmula con propiedades compatibles y condicionales
- **THEN** se calcula el resultado tipado desde los valores actuales sin modificar los originales

#### Scenario: Definición inválida o cíclica
- **WHEN** un AST excede límites, usa referencias ajenas, mezcla tipos/monedas o introduce un ciclo
- **THEN** se rechaza con error identificable y conserva definición/revisión previas

### Requirement: Dependencias y rollups coherentes

El sistema SHALL calcular conteo, suma, promedio, mínimo y máximo de propiedades relacionadas, incluyendo proyección inversa. MUST evaluar dependencias en un snapshot consistente, distinguir vacíos de errores y reflejar cambios de valores/relaciones sin publicar derivados viejos.

#### Scenario: Cambiar una fuente
- **WHEN** cambia una propiedad o relación usada por fórmula/rollup
- **THEN** la próxima consulta obtiene el cálculo vigente y conserva las revisiones originales de otras entidades

#### Scenario: Vacío y fallo de cálculo
- **WHEN** faltan valores o una dependencia falla por división por cero, precisión, overflow o referencia inválida
- **THEN** se devuelve vacío o un error explícito según la regla documentada, nunca un importe ficticio

### Requirement: Dinero exacto y reparto reproducible

El sistema MUST operar decimales sin floats monetarios, controlar precisión/moneda y repartir unidades mínimas ponderadas por restos mayores con desempate por ID estable. SHALL conservar el total de la fixture de Bs 145,69 y seis asientos.

#### Scenario: Suscripción y recálculo
- **WHEN** cinco miembros tienen pesos 2/1/1/1/1 y costo BOB 145,69 y luego se edita el costo
- **THEN** resultan 48,57 y cuatro 24,28, el total coincide y la siguiente lectura recalcula sin IA

### Requirement: Fórmulas compartidas y recuperables

CLI/escritorio SHALL definir fórmulas con revisión de colección y mostrar valores/errores derivados sin editarlos como originales. MUST conservar buffers ante conflicto y restaurar definiciones mediante backups/migración recuperable.

#### Scenario: Uso offline y restauración
- **WHEN** se guarda un AST por escritorio, CLI consulta y se restaura un backup sin red
- **THEN** las superficies calculan resultados equivalentes y las definiciones/originales se conservan
