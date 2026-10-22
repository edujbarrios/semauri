# 02 · Variables y tipos

**Objetivo:** Guardar información en nombres y distinguir números, texto y booleanos.

## Concepto

`let nombre be valor.` crea una vinculación inmutable: no puedes modificarla con otra asignación. Los literales entre comillas son texto; los números no llevan comillas; `true` y `false` son booleanos.

## Programa de ejemplo

Abre [`main.sema`](./main.sema):

```text
let nombre be "Ana".
let edad be 18.
let aprendiendo be true.

print nombre.
print edad.
print aprendiendo.
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/es/02_variables_y_tipos/main.sema
semauri run --allow console_write learn_to_code/es/02_variables_y_tipos/main.sema
```

**Salida esperada en la consola:**

```text
Ana
18
true
```

## Tu reto

Crea `let ciudad be "Madrid".` y muestra su valor. ¿Qué cambia si escribes el número 18 entre comillas?

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
