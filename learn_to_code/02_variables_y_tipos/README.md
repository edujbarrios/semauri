# 02 · Variables y tipos

**Objetivo:** Guardar información en nombres y distinguir números, texto y booleanos.

## Concepto

`Let nombre be valor.` crea una vinculación inmutable: no puedes modificarla con otra asignación. Los literales entre comillas son texto; los números no llevan comillas; `true` y `false` son booleanos.

## Programa de ejemplo

Abre [`main.sema`](./main.sema):

```text
Let nombre be "Ana".
Let edad be 18.
Let aprendiendo be true.

Print nombre.
Print edad.
Print aprendiendo.
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/02_variables_y_tipos/main.sema
semauri run --allow console_write learn_to_code/02_variables_y_tipos/main.sema
```

**Salida esperada en la consola:**

```text
Ana
18
true
```

## Tu reto

Crea `Let ciudad be "Madrid".` y muestra su valor. ¿Qué cambia si escribes el número 18 entre comillas?

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
