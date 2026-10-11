# 01 · Tu primer programa

**Objetivo:** Comprender qué es una instrucción y cómo imprimir información en la consola.

## Concepto

Cada línea terminada en punto es una instrucción. `Print` muestra un texto o valor. El código fuente se guarda como `.sema`.

## Programa de ejemplo

Abre [`main.sema`](./main.sema):

```text
# Un programa puede tener comentarios.
Print "Hola desde Semauri".
Print 2026.
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/01_hola_mundo/main.sema
semauri run --allow console_write learn_to_code/01_hola_mundo/main.sema
```

**Salida esperada en la consola:**

```text
Hola desde Semauri
2026
```

## Tu reto

Cambia el saludo para que muestre tu nombre. Después añade otra instrucción que imprima tu número favorito.

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
