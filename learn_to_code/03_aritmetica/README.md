# 03 · Aritmética y precedencia

**Objetivo:** Aprender expresiones, operadores y paréntesis.

## Concepto

Se admiten `plus`, `minus`, `times`, `divided by` y `modulo` (resto). La multiplicación tiene más prioridad que la suma; los paréntesis cambian el orden. La división por cero se rechaza.

## Programa de ejemplo

Abre [`main.sema`](./main.sema):

```text
Let subtotal be 10 plus 5 times 2.
Let agrupado be (10 plus 5) times 2.
Let resto be 17 modulo 3.

Print subtotal.
Print agrupado.
Print resto.
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/03_aritmetica/main.sema
semauri run --allow console_write learn_to_code/03_aritmetica/main.sema
```

**Salida esperada en la consola:**

```text
20
30
2
```

## Tu reto

Calcula el precio de tres artículos de 8 unidades cada uno más un envío de 5. Utiliza paréntesis para comparar dos cálculos.

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
