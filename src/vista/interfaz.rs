use gtk::prelude::*;
use gtk::{
    self, Application, ApplicationWindow, Box as GtkBox, Button, Grid, HeaderBar, Image, Label,
    Orientation, ScrolledWindow, Separator, ToggleButton,
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
    pub fn iniciar(&self, cantidad_rolas: i64) {
        self.aplicacion
            .connect_activate(move |application| Self::construir_ui(application, cantidad_rolas));
        self.aplicacion.run();
    }

    fn construir_ui(application: &Application, cantidad_rolas: i64) {
        let header = Self::construir_header(cantidad_rolas);
        header.pack_end(&Self::construir_botones_encabezado());

        let contenido = GtkBox::new(Orientation::Horizontal, 0);
        let izquierda = Self::construir_panel_izquierdo();
        let derecha = Self::construir_panel_derecho();
        let separador = Separator::new(Orientation::Vertical);

        contenido.append(&izquierda);
        contenido.append(&separador);
        contenido.append(&derecha);

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

    fn construir_header(cantidad_rolas: i64) -> HeaderBar {
        let header = HeaderBar::new();
        let titulo_header = GtkBox::new(Orientation::Vertical, 0);
        let titulo = Label::new(Some("Rolas"));
        titulo.add_css_class("title");
        let subtitulo = Label::new(Some(&format!("{cantidad_rolas} rolas")));
        subtitulo.add_css_class("subtitle");
        titulo_header.append(&titulo);
        titulo_header.append(&subtitulo);
        header.set_title_widget(Some(&titulo_header));
        header
    }

    fn construir_botones_encabezado() -> GtkBox {
        let controles = GtkBox::new(Orientation::Horizontal, 8);
        let botones_vista = GtkBox::new(Orientation::Horizontal, 0);
        botones_vista.add_css_class("linked");

        let boton_tabla = ToggleButton::new();
        boton_tabla.set_child(Some(&Image::from_icon_name("view-list-symbolic")));
        boton_tabla.set_tooltip_text(Some("Vista de tabla"));
        boton_tabla.set_active(true);

        let boton_alternativo = ToggleButton::new();
        boton_alternativo.set_child(Some(&Image::from_icon_name("view-grid-symbolic")));
        boton_alternativo.set_tooltip_text(Some("Vista alternativa (próximamente)"));
        boton_alternativo.set_group(Some(&boton_tabla));
        boton_alternativo.set_sensitive(false);

        botones_vista.append(&boton_tabla);
        botones_vista.append(&boton_alternativo);

        let boton_buscar = Button::from_icon_name("system-search-symbolic");
        boton_buscar.set_tooltip_text(Some("Buscar rolas (próximamente)"));
        boton_buscar.set_sensitive(false);

        controles.append(&botones_vista);
        controles.append(&boton_buscar);
        controles
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

    fn construir_panel_derecho() -> GtkBox {
        let panel = GtkBox::new(Orientation::Vertical, 0);
        panel.set_hexpand(true);
        panel.set_vexpand(true);
        panel.set_margin_start(16);
        panel.set_margin_end(16);
        panel.set_margin_top(12);
        panel.set_margin_bottom(12);
        panel.append(&Self::construir_tabla_canciones());
        panel
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
