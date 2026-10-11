# 10 · Tu primera página web

**Objetivo:** Generar HTML declarativamente con un dominio semántico.

## Concepto

Semauri puede transformar un programa en HTML con `build`. `Create a web`, `Add a button` y `Set the color ...` pertenecen al dominio Web. **No uses `run`** aquí: `build` genera el documento sin efectos externos.

## Programa de ejemplo

Abre [`main.sema`](./main.sema):

```text
Create a web called Mi Primera Pagina.
Add a button called Empezar.
Set the color of the button called Empezar to blue.
```

## Compruébalo

Desde la raíz del repositorio:

```sh
semauri check learn_to_code/10_crea_una_web/main.sema
semauri build learn_to_code/10_crea_una_web/main.sema
```

**Resultado esperado:** Genera HTML con un botón Empezar de color azul. El contenido generado aparece en la salida de `build`.

## Tu reto

Añade otro botón llamado Siguiente con color verde. Ejecuta `build` y busca las dos etiquetas `<button>` en el HTML generado.

Cuando termines, pasa de nuevo `check`. Si obtienes un error, revisa el mensaje y experimenta cambiando una sola cosa cada vez.

[← Volver al índice del curso](../README.md)
