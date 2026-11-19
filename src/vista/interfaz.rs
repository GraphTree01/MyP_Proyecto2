use crate::modelo::rola::Rolita;
use gtk::prelude::*;
use gtk::{
    self, Application, ApplicationWindow, Box as GtkBox, Button, Entry, FileDialog, Grid, HeaderBar,
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
    pub fn iniciar<F, B>(&self, rolas: Vec<Rolita>, al_minar: F, al_buscar: B)
    where
        F: Fn(PathBuf) -> Result<Vec<Rolita>, String> + Clone + 'static,
        B: Fn(String) -> Result<Vec<Rolita>, String> + Clone + 'static,
    {
        self.aplicacion.connect_activate(move |application| {
            Self::construir_ui(
                application,
                rolas.clone(),
                al_minar.clone(),
                al_buscar.clone(),
            )
        });
        self.aplicacion.run();
    }

    fn construir_ui<F, B>(application: &Application, rolas: Vec<Rolita>, al_minar: F, al_buscar: B)
    where
        F: Fn(PathBuf) -> Result<Vec<Rolita>, String> + 'static,
        B: Fn(String) -> Result<Vec<Rolita>, String> + 'static,
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
            Rc::new(al_buscar),
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
        al_buscar: Rc<dyn Fn(String) -> Result<Vec<Rolita>, String>>,
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

        controles.append(&botones_vista);
        let control_busqueda = Self::construir_boton_busqueda(
            al_buscar,
            subtitulo.clone(),
            mensaje.clone(),
            tabla.clone(),
        );
        controles.append(&control_busqueda);

        let boton_minar = Button::from_icon_name("folder-open-symbolic");
        boton_minar.set_tooltip_text(Some("Minar carpeta de música"));
        controles.append(&boton_minar);

        Self::conectar_boton_minar(&boton_minar, window, al_minar, subtitulo, mensaje, tabla);

        controles
    }

    fn construir_boton_busqueda(
        al_buscar: Rc<dyn Fn(String) -> Result<Vec<Rolita>, String>>,
        subtitulo: Label,
        mensaje: Label,
        tabla: Grid,
    ) -> GtkBox {
        let boton = GtkBox::new(Orientation::Horizontal, 0);
        let expandir_boton = Button::from_icon_name("system-search-symbolic");
        expandir_boton.set_tooltip_text(Some("Buscar"));

        let barra_busqueda = GtkBox::new(Orientation::Horizontal, 4);
        let entrada = Entry::new();
        entrada.set_placeholder_text(Some("Buscar (ej. title contiene \"love\")"));
        entrada.set_width_chars(28);
        let boton_ejecutar = Button::from_icon_name("system-search-symbolic");
        boton_ejecutar.set_tooltip_text(Some("Buscar"));
        let boton_cerrar = Button::from_icon_name("window-close-symbolic");
        boton_cerrar.set_tooltip_text(Some("Cerrar"));

        barra_busqueda.append(&entrada);
        barra_busqueda.append(&boton_cerrar);
        barra_busqueda.append(&boton_ejecutar);
        boton.append(&expandir_boton);

        let boton_abrir = boton.clone();
        let barra_abrir = barra_busqueda.clone();
        let boton_expandido = expandir_boton.clone();
        let entrada_abrir = entrada.clone();
        expandir_boton.connect_clicked(move |_| {
            boton_abrir.remove(&boton_expandido);
            boton_abrir.append(&barra_abrir);
            entrada_abrir.grab_focus();
        });

        let buscar = Rc::clone(&al_buscar);
        let entrada_buscar = entrada.clone();
        let subtitulo_buscar = subtitulo.clone();
        let mensaje_buscar = mensaje.clone();
        let tabla_buscar = tabla.clone();
        boton_ejecutar.connect_clicked(move |_| {
            Self::ejecutar_busqueda(
                entrada_buscar.text().to_string(),
                &*buscar,
                &subtitulo_buscar,
                &mensaje_buscar,
                &tabla_buscar,
            );
        });

        let buscar_enter = Rc::clone(&al_buscar);
        let subtitulo_enter = subtitulo.clone();
        let mensaje_enter = mensaje.clone();
        let tabla_enter = tabla.clone();
        entrada.connect_activate(move |entrada| {
            Self::ejecutar_busqueda(
                entrada.text().to_string(),
                &*buscar_enter,
                &subtitulo_enter,
                &mensaje_enter,
                &tabla_enter,
            );
        });

        let buscar_cerrar = Rc::clone(&al_buscar);
        let subtitulo_cerrar = subtitulo;
        let mensaje_cerrar = mensaje;
        let tabla_cerrar = tabla;
        let control_cerrar = boton.clone();
        let barra_cerrar = barra_busqueda.clone();
        let boton_abrir_cerrar = expandir_boton;
        boton_cerrar.connect_clicked(move |_| {
            if Self::ejecutar_busqueda(
                String::new(),
                &*buscar_cerrar,
                &subtitulo_cerrar,
                &mensaje_cerrar,
                &tabla_cerrar,
            ) {
                entrada.set_text("");
                control_cerrar.remove(&barra_cerrar);
                control_cerrar.append(&boton_abrir_cerrar);
                boton_abrir_cerrar.grab_focus();
            }
        });
        boton
    }

    fn ejecutar_busqueda(
        expresion: String,
        al_buscar: &dyn Fn(String) -> Result<Vec<Rolita>, String>,
        subtitulo: &Label,
        mensaje: &Label,
        tabla: &Grid,
    ) -> bool {
        match al_buscar(expresion.clone()) {
            Ok(rolas) => {
                actualizar_tabla(tabla, &rolas);
                if expresion.trim().is_empty() {
                    subtitulo.set_text(&format!("{} rolas", rolas.len()));
                    mensaje.set_text("");
                } else {
                    subtitulo.set_text(&format!("{} resultados", rolas.len()));
                    if rolas.is_empty() {
                        mensaje.set_text("No se encontraron rolas.");
                    } else {
                        mensaje.set_text(&format!("{} rolas encontradas.", rolas.len()));
                    }
                }
                true
            }
            Err(error) => {
                mensaje.set_text(&format!("Error en la búsqueda: {error}"));
                false
            }
        }
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
