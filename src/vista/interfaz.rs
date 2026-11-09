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
        let header = Self::construir_header();
        let button = Button::with_label("Cambiar derecha");
        header.pack_end(&button);

        let contenido = GtkBox::new(Orientation::Horizontal, 0);
        let izquierda = Self::construir_panel_izquierdo();
        let (derecha, mensaje) = Self::construir_panel_derecho();
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
            .default_width(1000)
            .default_height(650)
            .build();

        window.set_titlebar(Some(&header));
        window.set_child(Some(&contenido));
        window.present();
    }

    fn construir_header() -> HeaderBar {
        let header = HeaderBar::new();
        let titulo_header = GtkBox::new(Orientation::Vertical, 0);
        let titulo = Label::new(Some("Rolas"));
        titulo.add_css_class("title");
        let subtitulo = Label::new(Some("0 rolas"));
        subtitulo.add_css_class("subtitle");
        titulo_header.append(&titulo);
        titulo_header.append(&subtitulo);
        header.set_title_widget(Some(&titulo_header));
        header
    }

    fn construir_panel_izquierdo() -> GtkBox {
        let panel = GtkBox::new(Orientation::Vertical, 10);
        panel.set_size_request(260, -1);
        panel.set_margin_start(16);
        panel.set_margin_end(16);
        panel.set_margin_top(16);
        panel.append(&Label::new(Some("Panel izquierdo")));
        panel
    }

    fn construir_panel_derecho() -> (GtkBox, Label) {
        let panel = GtkBox::new(Orientation::Vertical, 0);
        panel.set_hexpand(true);
        panel.set_vexpand(true);
        panel.set_margin_start(16);
        panel.set_margin_end(16);
        panel.set_margin_top(12);
        panel.set_margin_bottom(12);
        panel.append(&Self::construir_tabla_canciones());

        let mensaje = Label::new(Some("No hay canciones cargadas"));
        panel.append(&mensaje);

        (panel, mensaje)
    }

    fn construir_tabla_canciones() -> ScrolledWindow {
        let tabla = Grid::new();
        tabla.set_column_spacing(16);
        tabla.set_row_spacing(0);
        tabla.add_css_class("view");

        for (columna, titulo) in ["Title", "Performer", "Album", "Genre"].iter().enumerate() {
            agregar_celda(&tabla, titulo, columna as i32, 0, true);
        }

        let separador = Separator::new(Orientation::Horizontal);
        tabla.attach(&separador, 0, 1, 4, 1);

        ScrolledWindow::builder()
            .child(&tabla)
            .vexpand(true)
            .hexpand(true)
            .build()
    }
}

fn agregar_celda(tabla: &Grid, texto: &str, columna: i32, fila: i32, encabezado: bool) {
    let etiqueta = Label::new(Some(texto));
    etiqueta.set_xalign(0.0);
    etiqueta.set_hexpand(true);
    etiqueta.set_width_chars(match columna {
        0 => 24,
        1 => 20,
        2 => 26,
        _ => 14,
    });
    etiqueta.set_max_width_chars(match columna {
        0 => 28,
        1 => 24,
        2 => 30,
        _ => 16,
    });
    etiqueta.set_ellipsize(gtk::pango::EllipsizeMode::End);
    etiqueta.set_margin_start(8);
    etiqueta.set_margin_end(8);
    etiqueta.set_margin_top(8);
    etiqueta.set_margin_bottom(8);
    if encabezado {
        etiqueta.add_css_class("heading");
    }
    tabla.attach(&etiqueta, columna, fila, 1, 1);
}
