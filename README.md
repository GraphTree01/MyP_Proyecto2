# MyP_Proyecto2

Una base de datos musical con una interfaz gráfica

## Funcionalidades

Permite minar archivos MP3, mostrar los resultados en forma de tabla o recuadros y editar la información de las canciones, álbumes e intérpretes.

## Requisitos

- Rust y Cargo.
- GTK 4.10 o posterior y sus dependencias de desarrollo.

## ¿Cómo ejecutar?

Desde la raíz del proyecto:

```sh
cargo run
```

La aplicación crea o abre `rolas.db` en el directorio desde donde se ejecuta.
Mantén ese archivo para conservar las rolas minadas entre ejecuciones.

## Guía de uso

1. Pulsa el botón con un icono de carpeta para elegir el directorio a minar. La
   aplicación recorre ese directorio y registra los archivos MP3 que encuentra.
2. Usa los botones de vista para alternar la manera de ver tus canciones; en forma de tabla ó recuadros.
3. Selecciona una fila o un recuadro para ver en el panel izquierdo la portada,
   el álbum, el intérprete y el título de la canción.
4. Pulsa el botón de búsqueda para buscar una canción por sus diferentes campos en la base de datos.

El grupo de cuatro botones de la izquierda del encabezado permiten 
editar el intérprete, el álbum o el título de la canción seleccionada.
Las opciones de edición requieren que primero se seleccione una canción. El botón
«+» permite crear a una persona independiente, si se selecciona una canción hecha
por una banda, es posible agregar a dicha persona a 
ese grupo. Todo lo anteriormente descrito solo ocurre en la base de datos.

## Opciones de búsqueda

Las búsquedas usan expresiones con la forma `campo operador valor`.

| Campo | Tipo | Ejemplo |
|---|---|---|
| `title` | Texto: título de la canción | `title contiene "love"` |
| `performer` | Texto: intérprete | `performer = "The Beatles"` |
| `album` | Texto: álbum | `album contiene "Abbey"` |
| `genre` | Texto: género | `genre != "Pop"` |
| `year` | Entero: año de la canción | `year >= 2000` |
| `track` | Entero: número de pista | `track = 3` |

Para campos de texto se permiten `=` (igual), `!=` (distinto) y
`contiene` (coincide con una parte del texto). El texto debe ir entre comillas
dobles. Para campos numéricos se permiten `=`, `!=`, `<>`, `<`, `<=`, `>` y
`>=`; el valor debe ser un entero.

Puedes combinar condiciones con `AND`, `OR` y `NOT`, y agruparlas con
paréntesis. Por ejemplo:

```text
title contiene "love" AND (performer = "The Beatles" OR genre = "Rock")
year >= 1990 AND year <= 1999
NOT genre = "Pop"
album contiene "live" OR title contiene "live"
```