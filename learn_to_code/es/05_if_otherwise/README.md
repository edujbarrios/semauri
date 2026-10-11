# 05 · Decisiones: If y Otherwise

**Objetivo:** Ejecutar una rama distinta según una condición.

## Concepto

`If condicion:` abre un bloque; `Otherwise:` ofrece una alternativa; `End.` cierra la estructura. La condición debe ser booleana. En estos ejemplos el compilador puede decidir la rama con valores conocidos.

## Programa de ejemplo

Abre [`main.sema`](./main.sema):

```text
Let nota be 7.

If nota is greater than or equal to 5:
  Print "Aprobado".
Otherwise:
  Print "Hay que practicar".
End.
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/es/05_if_otherwise/main.sema
semauri run --allow console_write learn_to_code/es/05_if_otherwise/main.sema
```

**Salida esperada en la consola:**

```text
Aprobado
```

## Tu reto

Cambia la nota a 3 y comprueba qué rama se elige. Añade otra condición con `and` para pedir nota y asistencia.

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
