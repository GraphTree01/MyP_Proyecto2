use gtk::prelude::*;
use gtk::{
    self, Application, ApplicationWindow, Box as GtkBox, Button, Grid, HeaderBar, Label,
    Orientation, ScrolledWindow, Separator,
};

const APP_ID: &str = "com.proyecto_dos.myp";

/// Representa la interfaz gráfica principal de la aplicación.
pub struct Vista {
    aplicacion: Application,
}

impl Vista {
    /// Crea una vista con una aplicación GTK configurada.
    pub fn nueva() -> Self {
        let aplicacion = Application::builder().application_id(APP_ID).build();
        Self { aplicacion }
    }

    /// Inicia el ciclo de vida de la interfaz gráfica GTK.
    pub fn iniciar(&self) {
        self.aplicacion.connect_activate(Self::construir_ui);
        self.aplicacion.run();
    }

    fn construir_ui(application: &Application) {
        let header = HeaderBar::new();

        let button = Button::with_label("Cambiar derecha");
        header.pack_end(&button);

        let contenido = GtkBox::new(Orientation::Horizontal, 200);

        let izquierda = GtkBox::new(Orientation::Vertical, 10);
        izquierda.append(&Label::new(Some("Panel izquierdo")));

        let derecha = GtkBox::new(Orientation::Vertical, 0);
        derecha.set_hexpand(true);
        derecha.set_vexpand(true);

        let encabezado = Grid::new();
        encabezado.set_column_homogeneous(true);
        encabezado.set_column_spacing(12);
        agregar_celda(&encabezado, "Title", 0, 0);
        agregar_celda(&encabezado, "Performer", 1, 0);
        agregar_celda(&encabezado, "Album", 2, 0);
        agregar_celda(&encabezado, "Genre", 3, 0);
        derecha.append(&encabezado);

        let canciones = Grid::new();
        canciones.set_column_homogeneous(true);
        canciones.set_column_spacing(12);

        let ventana_desplazable = ScrolledWindow::builder()
            .child(&canciones)
            .vexpand(true)
            .hexpand(true)
            .build();
        derecha.append(&ventana_desplazable);

        let mensaje = Label::new(Some("No hay canciones cargadas"));
        derecha.append(&mensaje);

        let separador = Separator::new(Orientation::Vertical);

        contenido.append(&izquierda);
        contenido.append(&separador);
        contenido.append(&derecha);

        button.connect_clicked(move |_| {
            mensaje.set_text("Contenido actualizado");
        });

        let window = ApplicationWindow::builder()
            .application(application)
            .title("Rolas")
            .default_width(500)
            .default_height(350)
            .build();

        window.set_titlebar(Some(&header));
        window.set_child(Some(&contenido));
        window.present();
    }
}

fn agregar_celda(tabla: &Grid, texto: &str, columna: i32, fila: i32) {
    let etiqueta = Label::new(Some(texto));
    etiqueta.set_xalign(0.0);
    tabla.attach(&etiqueta, columna, fila, 1, 1);
}
