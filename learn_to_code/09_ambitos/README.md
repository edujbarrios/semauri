# 09 · Ámbitos y nombres

**Objetivo:** Entender la visibilidad de las variables y el sombreado de nombres.

## Concepto

Los bloques crean ámbitos léxicos. Un `Let` dentro de `If` solo existe en ese bloque. Un nombre interno puede **sombrear** uno externo sin modificar su valor. Si intentas usar fuera una variable creada dentro, el compilador lo rechazará.

## Programa de ejemplo

Abre [`main.sema`](./main.sema):

```text
Let mensaje be "fuera".

If true:
  Let mensaje be "dentro".
  Print mensaje.
End.

Print mensaje.
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/09_ambitos/main.sema
semauri run --allow console_write learn_to_code/09_ambitos/main.sema
```

**Salida esperada en la consola:**

```text
dentro
fuera
```

## Tu reto

Añade `Let secreto be 7.` dentro del bloque e intenta imprimir `secreto` después de `End.`. Corrige el error moviendo su declaración.

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
