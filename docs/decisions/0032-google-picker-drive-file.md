# ADR 0032: Google Picker con `drive.file` en lugar de scopes restringidos

- Fecha: 22 de octubre de 2026
- Estado: adoptado para la integración inicial de Drive
- Ámbito: navegación y selección de archivos existentes

## Contexto

Noteka necesita que el usuario pueda navegar y asociar archivos de su Drive sin
otorgar acceso genérico a toda la cuenta. Google clasifica `drive.readonly` y
`drive.metadata.readonly` como scopes restringidos. Para distribuirlos se exige
verificación y una evaluación CASA anual de un tercero; para uso personal existe
una excepción, pero la app queda no verificada, con advertencias y límites.

`drive.file` es un scope no sensible: da acceso por archivo a lo que la app crea
o a lo que el usuario selecciona explícitamente. Google Picker permite mostrar la
interfaz de Drive y devolver los `picked_file_ids` sin que la app enumere el
resto.

## Decisión

1. La integración inicial usa **Google Picker API + `drive.file`**. No se solicitan
   `drive`, `drive.readonly` ni `drive.metadata.readonly`.
2. `google pick` abre una autorización separada con sólo `drive.file`,
   `prompt=consent` y `trigger_onepick=true`; el callback loopback devuelve
   `picked_file_ids`.
3. La selección y su token se guardan en una sesión Keychain separada
   (`google-drive-picker`) para no pisar la identidad de `google login`. El máximo
   inicial es 20 archivos.
4. `google selected` consulta sólo metadatos de los IDs elegidos
   (`id/name/mimeType/modifiedTime/webViewLink`), con refresh si expiró.
5. Cancelar el Picker no reemplaza la selección anterior. No hay descarga,
   escritura ni edición de archivos.
6. La UI, las asociaciones universales en SQLite y los previews Docs/Sheets son
   un tramo posterior; esta decisión no los declara implementados.

## Alternativas

- **Scope restringido + navegador propio**: permite listar todo el Drive, pero
  exige verificación/CASA para distribución y deja la app no verificada en uso
  personal. Se descarta para el recorte inicial.
- **Enlaces manuales**: el usuario pega una URL o ID; no permite navegar y el
  acceso con `drive.file` no está garantizado. Se descarta.
- **Service account**: no representa el consentimiento personal ni el acceso a
  archivos del usuario. Se descarta.

## Consecuencias

- El usuario navega en el Picker de Google, no en una lista propia de Noteka.
- No hay costo de verificación/CASA para este flujo; Picker API y Drive API son
  gratuitas dentro de la cuota estándar.
- Si más adelante se necesita un explorador propio de todo el Drive, se evalúa
  como modo avanzado con scope restringido y su costo de verificación.
- La selección Picker es una concesión de acceso a la app; revocarla requiere
  quitarla desde la cuenta de Google o volver a seleccionar.
- El token Picker sólo tiene `drive.file`; no se usa para identidad.

## Evidencia

`docs/verification-google-drive.md`. El flujo real se probó con un archivo
existente y una cancelación; no se versionan IDs, nombres ni tokens.
