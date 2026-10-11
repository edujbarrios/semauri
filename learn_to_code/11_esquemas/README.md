# 11 · Datos estructurados y JSON Schema

**Objetivo:** Describir la estructura esperada de los datos.

## Concepto

El dominio Structured Data permite `Create a schema`, `Add a field` y propiedades tipadas. `build` produce JSON Schema. Un esquema valida la **forma** de los datos: no guarda registros ni realiza operaciones de archivos.

## Programa de ejemplo

Abre [`main.sema`](./main.sema):

```text
Create a schema called Alumno.
Add a field called Nombre.
Set the datatype of the field called Nombre to "string".
Set the required of the field called Nombre to true.
Add a field called Edad.
Set the datatype of the field called Edad to "integer".
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/11_esquemas/main.sema
semauri build learn_to_code/11_esquemas/main.sema
```

**Resultado esperado:** Genera JSON Schema con Nombre obligatorio y Edad entero. El contenido generado aparece en la salida de `build`.

## Tu reto

Añade un campo Curso de tipo `string` y márcalo como obligatorio. Busca la propiedad `required` en el JSON generado.

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
