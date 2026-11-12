use crate::modelo::rola::Rolita;
use gtk::prelude::*;
use gtk::{
    self, Application, ApplicationWindow, Box as GtkBox, Button, FileDialog, Grid, HeaderBar,
    Image, Label, Orientation, ScrolledWindow, Separator, ToggleButton,
};
use std::path::PathBuf;
use std::rc::Rc;

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
    pub fn iniciar<F>(&self, rolas: Vec<Rolita>, al_minar: F)
    where
        F: Fn(PathBuf) -> Result<Vec<Rolita>, String> + Clone + 'static,
    {
        self.aplicacion.connect_activate(move |application| {
            Self::construir_ui(application, rolas.clone(), al_minar.clone())
        });
        self.aplicacion.run();
    }

    fn construir_ui<F>(application: &Application, rolas: Vec<Rolita>, al_minar: F)
    where
        F: Fn(PathBuf) -> Result<Vec<Rolita>, String> + 'static,
    {
        let contenido = GtkBox::new(Orientation::Horizontal, 0);
        let izquierda = Self::construir_panel_izquierdo();
        let (derecha, tabla, mensaje) = Self::construir_panel_derecho(&rolas);
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

        let (header, subtitulo) = Self::construir_header(rolas.len());
        header.pack_end(&Self::construir_botones_encabezado(
            &window,
            Rc::new(al_minar),
            subtitulo,
            mensaje,
            tabla,
        ));
        window.set_titlebar(Some(&header));
        window.set_child(Some(&contenido));
        window.present();
    }

    fn construir_header(cantidad_rolas: usize) -> (HeaderBar, Label) {
        let header = HeaderBar::new();
        let titulo_header = GtkBox::new(Orientation::Vertical, 0);
        let titulo = Label::new(Some("Rolas"));
        titulo.add_css_class("title");
        let subtitulo = Label::new(Some(&format!("{cantidad_rolas} rolas")));
        subtitulo.add_css_class("subtitle");
        titulo_header.append(&titulo);
        titulo_header.append(&subtitulo);
        header.set_title_widget(Some(&titulo_header));
        (header, subtitulo)
    }

    fn construir_botones_encabezado(
        window: &ApplicationWindow,
        al_minar: Rc<dyn Fn(PathBuf) -> Result<Vec<Rolita>, String>>,
        subtitulo: Label,
        mensaje: Label,
        tabla: Grid,
    ) -> GtkBox {
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

        let boton_minar = Button::from_icon_name("folder-open-symbolic");
        boton_minar.set_tooltip_text(Some("Minar carpeta de música"));
        controles.append(&boton_minar);

        Self::conectar_boton_minar(&boton_minar, window, al_minar, subtitulo, mensaje, tabla);

        controles
    }

    fn conectar_boton_minar(
        boton: &Button,
        window: &ApplicationWindow,
        al_minar: Rc<dyn Fn(PathBuf) -> Result<Vec<Rolita>, String>>,
        subtitulo: Label,
        mensaje: Label,
        tabla: Grid,
    ) {
        let padre = window.clone();
        boton.connect_clicked(move |_| {
            let selector = FileDialog::new();
            selector.set_title("Seleccionar carpeta de música");
            selector.set_accept_label(Some("Minar"));
            selector.set_modal(true);
            let al_minar = Rc::clone(&al_minar);
            let subtitulo = subtitulo.clone();
            let mensaje = mensaje.clone();
            let tabla = tabla.clone();
            selector.select_folder(
                Some(&padre),
                None::<&gtk::gio::Cancellable>,
                move |resultado| match resultado {
                    Ok(carpeta) => {
                        if let Some(directorio) = carpeta.path() {
                            mensaje.set_text("Minando archivos MP3...");
                            match al_minar(directorio) {
                                Ok(rolas) => {
                                    actualizar_tabla(&tabla, &rolas);
                                    subtitulo.set_text(&format!("{} rolas", rolas.len()));
                                    mensaje.set_text("Minado completado.");
                                }
                                Err(error) => {
                                    mensaje.set_text(&format!("Error al minar: {error}"));
                                }
                            }
                        } else {
                            mensaje.set_text("No se pudo obtener la ruta de la carpeta.");
                        }
                    }
                    Err(error) if error.matches(gtk::gio::IOErrorEnum::Cancelled) => {}
                    Err(error) => mensaje.set_text(&format!(
                        "No se pudo abrir el selector de carpetas: {error}"
                    )),
                },
            );
        });
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

    fn construir_panel_derecho(rolas: &[Rolita]) -> (GtkBox, Grid, Label) {
        let panel = GtkBox::new(Orientation::Vertical, 0);
        panel.set_hexpand(true);
        panel.set_vexpand(true);
        panel.set_margin_start(16);
        panel.set_margin_end(16);
        panel.set_margin_top(12);
        panel.set_margin_bottom(12);
        let (tabla_desplazable, tabla) = Self::construir_tabla_canciones(rolas);
        panel.append(&tabla_desplazable);

        let mensaje = Label::new(Some(""));
        panel.append(&mensaje);
        (panel, tabla, mensaje)
    }

    fn construir_tabla_canciones(rolas: &[Rolita]) -> (ScrolledWindow, Grid) {
        let tabla = Grid::new();
        tabla.set_column_spacing(16);
        tabla.set_row_spacing(0);
        tabla.add_css_class("view");

        actualizar_tabla(&tabla, rolas);

        let desplazamiento = ScrolledWindow::builder()
            .child(&tabla)
            .vexpand(true)
            .hexpand(true)
            .build();
        (desplazamiento, tabla)
    }
}

fn actualizar_tabla(tabla: &Grid, rolas: &[Rolita]) {
    while let Some(hijo) = tabla.first_child() {
        tabla.remove(&hijo);
    }

    for (columna, titulo) in ["Title", "Performer", "Album", "Genre"].iter().enumerate() {
        agregar_celda(tabla, titulo, columna as i32, 0, true);
    }

    let separador = Separator::new(Orientation::Horizontal);
    tabla.attach(&separador, 0, 1, 4, 1);

    for (indice, rola) in rolas.iter().enumerate() {
        let fila = (indice as i32) * 2 + 2;
        agregar_celda(tabla, &rola.titulo, 0, fila, false);
        agregar_celda(tabla, &rola.performer, 1, fila, false);
        agregar_celda(tabla, &rola.album, 2, fila, false);
        agregar_celda(tabla, rola.genero.as_deref().unwrap_or("—"), 3, fila, false);

        if indice + 1 < rolas.len() {
            let separador = Separator::new(Orientation::Horizontal);
            tabla.attach(&separador, 0, fila + 1, 4, 1);
        }
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
