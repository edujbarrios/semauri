# 08 · Repetición: For every

**Objetivo:** Ejecutar una instrucción para cada elemento de una lista.

## Concepto

`for every elemento in lista:` recorre una lista y usa `elemento` dentro del bloque. Por ahora los bucles son **estáticos**, evaluados a partir de colecciones conocidas durante la compilación, no un bucle general sobre datos de entrada dinámicos.

## Programa de ejemplo

Abre [`main.suri`](./main.suri):

```text
let niveles be a list of 1, 2, 3.

for every nivel in niveles:
  print nivel.
end.

print "Fin".
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/es/08_bucles/main.suri
semauri run --allow console_write learn_to_code/es/08_bucles/main.suri
```

**Salida esperada en la consola:**

```text
1
2
3
Fin
```

## Tu reto

Crea una lista de números diferentes y muestra el resultado de `nivel times 2` dentro del bucle.

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
