# Aprende a programar con Semauri

Curso **gratuito, en español y para principiantes**. No hace falta saber programar. Trabajaremos con la sintaxis real del compilador de Semauri **0.16.0** (lenguaje natural **controlado**, con palabras clave en inglés), no con pseudocódigo.

## Cómo empezar

1. [Instala Semauri](../docs/00_INSTALLATION.md). Comprueba que funciona con `semauri version`. También puedes usar `cargo run --` desde la raíz del repositorio si tienes Rust.
2. Descarga o clona este repositorio, o utiliza la carpeta `learn_to_code/` incluida en los paquetes nativos de la release 0.16.0.
3. Ejecuta los comandos desde la **raíz del repositorio**; si usas un paquete, puedes entrar en la carpeta que contiene `learn_to_code/`.

```sh
semauri check learn_to_code/01_hola_mundo/main.sema
semauri run --dry-run learn_to_code/01_hola_mundo/main.sema
semauri run --allow console_write learn_to_code/01_hola_mundo/main.sema
```

**Importante:** `check` revisa sintaxis y tipos, pero **no ejecuta**. `run --dry-run` muestra el plan sin efectos. `run --allow console_write` permite explícitamente escribir en la consola. Las lecciones 10 y 11 utilizan `semauri build` y no necesitan ejecutar operaciones.

## Itinerario (en orden)

| # | Lección | Qué aprendes | Cómo probar |
|---|---|---|---|
| 1 | [Tu primer programa](./01_hola_mundo/README.md) | Comprender qué es una instrucción y cómo imprimir información en la consola. | `run` |
| 2 | [Variables y tipos](./02_variables_y_tipos/README.md) | Guardar información en nombres y distinguir números, texto y booleanos. | `run` |
| 3 | [Aritmética y precedencia](./03_aritmetica/README.md) | Aprender expresiones, operadores y paréntesis. | `run` |
| 4 | [Booleanos y comparaciones](./04_booleanos/README.md) | Formular preguntas que devuelven verdadero o falso. | `run` |
| 5 | [Decisiones: If y Otherwise](./05_if_otherwise/README.md) | Ejecutar una rama distinta según una condición. | `run` |
| 6 | [Varias decisiones: Unless y Otherwise if](./06_unless_y_else_if/README.md) | Construir una cadena de condiciones sin repetir bloques anidados. | `run` |
| 7 | [Listas de valores](./07_listas/README.md) | Agrupar elementos del mismo tipo en una lista. | `run` |
| 8 | [Repetición: For every](./08_bucles/README.md) | Ejecutar una instrucción para cada elemento de una lista. | `run` |
| 9 | [Ámbitos y nombres](./09_ambitos/README.md) | Entender la visibilidad de las variables y el sombreado de nombres. | `run` |
| 10 | [Tu primera página web](./10_crea_una_web/README.md) | Generar HTML declarativamente con un dominio semántico. | `build` |
| 11 | [Datos estructurados y JSON Schema](./11_esquemas/README.md) | Describir la estructura esperada de los datos. | `build` |

Cada carpeta contiene un **README con explicación y reto**, más un `main.sema` ejecutable. Haz el reto modificando una copia del archivo, pasa `check` y después observa el resultado. No se requiere conexión a Internet para ejecutar las lecciones.

## Comandos útiles

```sh
# Ejemplo de una lección de consola (01 a 09)
semauri check learn_to_code/05_if_otherwise/main.sema
semauri run --allow console_write learn_to_code/05_if_otherwise/main.sema

# Lecciones de generación de archivos (10 y 11)
semauri check learn_to_code/10_crea_una_web/main.sema
semauri build learn_to_code/10_crea_una_web/main.sema
semauri build learn_to_code/11_esquemas/main.sema

# Ver la interpretación del compilador
semauri explain learn_to_code/05_if_otherwise/main.sema
```

Puedes aprender incluso sin autorizar efectos: usa `check`, `tokens`, `ast`, `hir`, `explain` y `build`. Si ves un diagnóstico como `S312` (variable no encontrada) o `S316` (condición no booleana), lee su ubicación en el archivo y consulta [la especificación](../docs/01_LANGUAGE.md).

## Qué no enseña aún este curso

Semauri sigue siendo **experimental**. Aún no hay funciones de usuario, entrada interactiva ni bucles sobre colecciones dinámicas. No copies ejemplos de Python o JavaScript esperando que su sintaxis funcione aquí. La diferencia es parte del aprendizaje: las reglas de los lenguajes son precisas.

## ¿Y después?

Mira los [ejemplos adicionales](../examples/), el [manual de tipos](../docs/02_TYPES.md) y la [guía del desarrollador](../docs/DEVELOPMENT.md). Para contribuir, consulta [CONTRIBUTING.md](../CONTRIBUTING.md).

**Mantenimiento:** los programas `main.sema` de este curso están cubiertos por tests automáticos en `tests/learning_course.rs` y se empaquetan en los seis artefactos nativos oficiales.
