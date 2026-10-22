# 07 · Listas de valores

**Objetivo:** Agrupar elementos del mismo tipo en una lista.

## Concepto

`a list of ...` crea una lista homogénea: todos sus elementos deben tener el mismo tipo. Puedes imprimirla completa. La versión actual no permite acceder a elementos por índice.

## Programa de ejemplo

Abre [`main.sema`](./main.sema):

```text
let puntuaciones be a list of 10, 20, 30.
let colores be a list of red, green, blue.

print puntuaciones.
print colores.
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/es/07_listas/main.sema
semauri run --allow console_write learn_to_code/es/07_listas/main.sema
```

**Salida esperada en la consola:**

```text
[10, 20, 30]
[red, green, blue]
```

## Tu reto

Crea una lista de tres textos. Intenta mezclar números y textos: `semauri check` debería mostrar un error de tipos.

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
