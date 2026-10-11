# 06 · Varias decisiones: Unless y Otherwise if

**Objetivo:** Construir una cadena de condiciones sin repetir bloques anidados.

## Concepto

`Unless condicion:` significa `If not condicion:`. En una cadena, `Otherwise if` o `Else if` evalúa la siguiente condición si no se ha elegido una rama anterior. Toda la cadena comparte un solo `End.`.

## Programa de ejemplo

Abre [`main.sema`](./main.sema):

```text
Let nota be 82.

Unless nota is greater than or equal to 50:
  Print "Suspenso".
Otherwise if nota is greater than or equal to 90:
  Print "Excelente".
Otherwise if nota is greater than or equal to 70:
  Print "Notable".
Otherwise:
  Print "Aprobado".
End.
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/es/06_unless_y_else_if/main.sema
semauri run --allow console_write learn_to_code/es/06_unless_y_else_if/main.sema
```

**Salida esperada en la consola:**

```text
Notable
```

## Tu reto

Prueba las notas 30, 65, 75 y 95. Escribe qué mensaje corresponde a cada una antes de ejecutar.

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
