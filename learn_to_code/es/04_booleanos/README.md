# 04 · Booleanos y comparaciones

**Objetivo:** Formular preguntas que devuelven verdadero o falso.

## Concepto

`is greater than`, `is less than`, `is equal to`, `and`, `or` y `not` forman expresiones booleanas. `and` exige ambas condiciones; `or` necesita una; `not` invierte el resultado.

## Programa de ejemplo

Abre [`main.sema`](./main.sema):

```text
Let edad be 20.
Let mayor be edad is greater than or equal to 18.
Let permiso be mayor and not false.

Print mayor.
Print permiso.
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/es/04_booleanos/main.sema
semauri run --allow console_write learn_to_code/es/04_booleanos/main.sema
```

**Salida esperada en la consola:**

```text
true
true
```

## Tu reto

Cambia la edad a 16 y observa el resultado. Después crea una comparación con `is less than`.

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
