use crate::modelo::metadatos::Metadatos;
use crate::modelo::rola::Rolita;
use gtk::prelude::*;
use gtk::{
    self, Application, ApplicationWindow, Box as GtkBox, Button, Entry, FileDialog, FlowBox, Frame,
    Grid, HeaderBar, Image, Label, Orientation, ScrolledWindow, Separator, Stack, ToggleButton,
};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

const APP_ID: &str = "com.proyecto_dos.myp";

#[derive(Clone)]
struct PanelResultados {
    vistas: Stack,
    tabla: Grid,
    recuadros: FlowBox,
    subtitulo: Label,
    mensaje: Label,
    detalle: PanelSeleccion,
    en_recuadros: Rc<Cell<bool>>,
    expresion_actual: Rc<RefCell<String>>,
    ruta_seleccionada: Rc<RefCell<Option<PathBuf>>>,
    boton_seleccionado: Rc<RefCell<Option<Button>>>,
}

#[derive(Clone)]
struct PanelSeleccion {
    caratula: Image,
    album: Label,
    performer: Label,
    titulo: Label,
    mensaje: Label,
}

impl PanelSeleccion {
    fn mostrar(&self, rola: &Rolita) {
        let caratula = match &rola.caratula {
            Some(caratula) => Ok(Some(caratula.clone())),
            None => Metadatos::leer_caratula(&rola.path),
        };
        match caratula {
            Ok(Some(bytes)) => {
                let bytes = gtk::glib::Bytes::from_owned(bytes);
                match gtk::gdk::Texture::from_bytes(&bytes) {
                    Ok(textura) => {
                        self.caratula.set_paintable(Some(&textura));
                        self.mensaje.set_text("");
                    }
                    Err(error) => {
                        self.mostrar_caratula_generica();
                        self.mensaje
                            .set_text(&format!("No se pudo mostrar la carátula: {error}"));
                    }
                }
            }
            Ok(None) => {
                self.mostrar_caratula_generica();
                self.mensaje.set_text("");
            }
            Err(error) => {
                self.mostrar_caratula_generica();
                self.mensaje
                    .set_text(&format!("No se pudo leer la carátula: {error}"));
            }
        }

        self.album.set_text(&rola.album);
        self.performer.set_text(&rola.performer);
        self.titulo.set_text(&rola.titulo);
    }

    fn limpiar(&self) {
        self.mostrar_caratula_generica();
        self.album.set_text("Álbum");
        self.performer.set_text("Intérprete");
        self.titulo.set_text("Canción");
        self.mensaje.set_text("Selecciona una rola");
    }

    fn mostrar_caratula_generica(&self) {
        self.caratula.set_paintable(None::<&gtk::gdk::Paintable>);
        self.caratula
            .set_icon_name(Some("audio-x-generic-symbolic"));
    }
}

impl PanelResultados {
    fn ejecutar_busqueda(
        &self,
        expresion: String,
        al_buscar: &dyn Fn(String, bool) -> Result<Vec<Rolita>, String>,
    ) -> bool {
        let incluir_caratulas = self.en_recuadros.get();
        match al_buscar(expresion.clone(), incluir_caratulas) {
            Ok(rolas) => {
                let seleccion = self.ruta_seleccionada.borrow().clone();
                if let Some(rola) = seleccion
                    .as_ref()
                    .and_then(|ruta| rolas.iter().find(|rola| &rola.path == ruta))
                {
                    self.detalle.mostrar(rola);
                } else if seleccion.is_some() {
                    *self.ruta_seleccionada.borrow_mut() = None;
                    self.boton_seleccionado.borrow_mut().take();
                    self.detalle.limpiar();
                }
                self.actualizar_vista(&rolas);
                *self.expresion_actual.borrow_mut() = expresion.clone();
                if expresion.trim().is_empty() {
                    self.subtitulo.set_text(&format!("{} rolas", rolas.len()));
                    self.mensaje.set_text("");
                } else {
                    self.subtitulo
                        .set_text(&format!("{} resultados", rolas.len()));
                    if rolas.is_empty() {
                        self.mensaje.set_text("No se encontraron rolas.");
                    } else {
                        self.mensaje
                            .set_text(&format!("{} rolas encontradas.", rolas.len()));
                    }
                }
                true
            }
            Err(error) => {
                self.mensaje
                    .set_text(&format!("Error en la búsqueda: {error}"));
                false
            }
        }
    }

    fn seleccionar(&self, rola: &Rolita, boton: &Button) {
        let anterior = self.boton_seleccionado.borrow_mut().take();
        if let Some(anterior) = anterior {
            anterior.remove_css_class("selected");
        }
        boton.add_css_class("selected");
        *self.boton_seleccionado.borrow_mut() = Some(boton.clone());
        *self.ruta_seleccionada.borrow_mut() = Some(rola.path.clone());
        self.detalle.mostrar(rola);
    }

    fn actualizar_vista(&self, rolas: &[Rolita]) {
        let anterior = self.boton_seleccionado.borrow_mut().take();
        if let Some(anterior) = anterior {
            anterior.remove_css_class("selected");
        }
        if self.en_recuadros.get() {
            actualizar_recuadros(&self.recuadros, rolas, self);
        } else {
            actualizar_tabla(&self.tabla, rolas, self);
        }
    }
}

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
        F: Fn(PathBuf) -> Result<(), String> + Clone + 'static,
        B: Fn(String, bool) -> Result<Vec<Rolita>, String> + Clone + 'static,
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
        F: Fn(PathBuf) -> Result<(), String> + 'static,
        B: Fn(String, bool) -> Result<Vec<Rolita>, String> + 'static,
    {
        let contenido = GtkBox::new(Orientation::Horizontal, 0);
        let (izquierda, detalle) = Self::construir_panel_izquierdo();
        let (derecha, vistas, tabla, recuadros, mensaje) = Self::construir_panel_derecho();
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
        let resultados = PanelResultados {
            vistas,
            tabla,
            recuadros,
            subtitulo,
            mensaje,
            detalle,
            en_recuadros: Rc::new(Cell::new(false)),
            expresion_actual: Rc::new(RefCell::new(String::new())),
            ruta_seleccionada: Rc::new(RefCell::new(None)),
            boton_seleccionado: Rc::new(RefCell::new(None)),
        };
        resultados.actualizar_vista(&rolas);
        header.pack_end(&Self::construir_botones_encabezado(
            &window,
            Rc::new(al_minar),
            Rc::new(al_buscar),
            resultados,
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
        al_minar: Rc<dyn Fn(PathBuf) -> Result<(), String>>,
        al_buscar: Rc<dyn Fn(String, bool) -> Result<Vec<Rolita>, String>>,
        resultados: PanelResultados,
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
        boton_alternativo.set_tooltip_text(Some("Vista de recuadros"));
        boton_alternativo.set_group(Some(&boton_tabla));

        botones_vista.append(&boton_tabla);
        botones_vista.append(&boton_alternativo);

        controles.append(&botones_vista);
        Self::conectar_botones_vista(
            &boton_tabla,
            &boton_alternativo,
            Rc::clone(&al_buscar),
            resultados.clone(),
        );

        let buscar_minar = Rc::clone(&al_buscar);
        let control_busqueda = Self::construir_boton_busqueda(al_buscar, resultados.clone());
        controles.append(&control_busqueda);

        let boton_minar = Button::from_icon_name("folder-open-symbolic");
        boton_minar.set_tooltip_text(Some("Minar carpeta de música"));
        controles.append(&boton_minar);

        Self::conectar_boton_minar(&boton_minar, window, al_minar, buscar_minar, resultados);

        controles
    }

    fn construir_boton_busqueda(
        al_buscar: Rc<dyn Fn(String, bool) -> Result<Vec<Rolita>, String>>,
        resultados: PanelResultados,
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
        let resultados_buscar = resultados.clone();
        boton_ejecutar.connect_clicked(move |_| {
            resultados_buscar.ejecutar_busqueda(entrada_buscar.text().to_string(), &*buscar);
        });

        let buscar_enter = Rc::clone(&al_buscar);
        let resultados_enter = resultados.clone();
        entrada.connect_activate(move |entrada| {
            resultados_enter.ejecutar_busqueda(entrada.text().to_string(), &*buscar_enter);
        });

        let buscar_cerrar = Rc::clone(&al_buscar);
        let resultados_cerrar = resultados;
        let control_cerrar = boton.clone();
        let barra_cerrar = barra_busqueda.clone();
        let boton_abrir_cerrar = expandir_boton;
        boton_cerrar.connect_clicked(move |_| {
            if resultados_cerrar.ejecutar_busqueda(String::new(), &*buscar_cerrar) {
                entrada.set_text("");
                control_cerrar.remove(&barra_cerrar);
                control_cerrar.append(&boton_abrir_cerrar);
                boton_abrir_cerrar.grab_focus();
            }
        });
        boton
    }

    fn conectar_botones_vista(
        boton_tabla: &ToggleButton,
        boton_recuadros: &ToggleButton,
        al_buscar: Rc<dyn Fn(String, bool) -> Result<Vec<Rolita>, String>>,
        resultados: PanelResultados,
    ) {
        for (boton, modo_recuadros) in [(boton_tabla, false), (boton_recuadros, true)] {
            let buscar = Rc::clone(&al_buscar);
            let resultados = resultados.clone();
            let tabla = boton_tabla.clone();
            let recuadros = boton_recuadros.clone();
            boton.connect_toggled(move |boton| {
                if !boton.is_active() || resultados.en_recuadros.get() == modo_recuadros {
                    return;
                }

                resultados.en_recuadros.set(modo_recuadros);
                resultados.vistas.set_visible_child_name(if modo_recuadros {
                    "recuadros"
                } else {
                    "tabla"
                });
                let expresion = resultados.expresion_actual.borrow().clone();
                if !resultados.ejecutar_busqueda(expresion, &*buscar) {
                    resultados.en_recuadros.set(!modo_recuadros);
                    tabla.set_active(!modo_recuadros);
                    recuadros.set_active(modo_recuadros);
                    resultados.vistas.set_visible_child_name(if modo_recuadros {
                        "tabla"
                    } else {
                        "recuadros"
                    });
                }
            });
        }
    }

    fn conectar_boton_minar(
        boton: &Button,
        window: &ApplicationWindow,
        al_minar: Rc<dyn Fn(PathBuf) -> Result<(), String>>,
        al_buscar: Rc<dyn Fn(String, bool) -> Result<Vec<Rolita>, String>>,
        resultados: PanelResultados,
    ) {
        let padre = window.clone();
        boton.connect_clicked(move |_| {
            let selector = FileDialog::new();
            selector.set_title("Seleccionar carpeta de música");
            selector.set_accept_label(Some("Minar"));
            selector.set_modal(true);
            let al_minar = Rc::clone(&al_minar);
            let resultados = resultados.clone();
            let buscar = Rc::clone(&al_buscar);
            selector.select_folder(
                Some(&padre),
                None::<&gtk::gio::Cancellable>,
                move |resultado| match resultado {
                    Ok(carpeta) => {
                        if let Some(directorio) = carpeta.path() {
                            resultados.mensaje.set_text("Minando archivos MP3...");
                            match al_minar(directorio) {
                                Ok(()) => {
                                    let expresion = resultados.expresion_actual.borrow().clone();
                                    if resultados.ejecutar_busqueda(expresion, &*buscar) {
                                        resultados.mensaje.set_text("Minado completado.");
                                    }
                                }
                                Err(error) => {
                                    resultados
                                        .mensaje
                                        .set_text(&format!("Error al minar: {error}"));
                                }
                            }
                        } else {
                            resultados
                                .mensaje
                                .set_text("No se pudo obtener la ruta de la carpeta.");
                        }
                    }
                    Err(error) if error.matches(gtk::gio::IOErrorEnum::Cancelled) => {}
                    Err(error) => resultados.mensaje.set_text(&format!(
                        "No se pudo abrir el selector de carpetas: {error}"
                    )),
                },
            );
        });
    }

    fn construir_panel_izquierdo() -> (GtkBox, PanelSeleccion) {
        let panel = GtkBox::new(Orientation::Vertical, 10);
        panel.set_size_request(288, -1);
        panel.set_hexpand(false);
        panel.set_halign(gtk::Align::Fill);
        panel.set_margin_start(16);
        panel.set_margin_end(16);
        panel.set_margin_top(16);
        panel.set_margin_bottom(16);

        let caratula = Image::from_icon_name("audio-x-generic-symbolic");
        caratula.set_pixel_size(220);
        caratula.set_size_request(220, 220);
        caratula.set_halign(gtk::Align::Center);
        caratula.set_valign(gtk::Align::Center);
        caratula.set_hexpand(false);
        caratula.set_vexpand(false);
        let espacio_caratula = Frame::new(None);
        espacio_caratula.set_size_request(220, 220);
        espacio_caratula.set_halign(gtk::Align::Center);
        espacio_caratula.set_margin_top(24);
        espacio_caratula.set_margin_bottom(12);
        espacio_caratula.set_child(Some(&caratula));
        panel.append(&espacio_caratula);

        let detalles = Grid::new();
        detalles.set_column_homogeneous(true);
        detalles.set_row_spacing(0);
        detalles.add_css_class("view");
        let titulo = Self::construir_fila_detalle("audio-x-generic-symbolic", "Canción");
        let performer = Self::construir_fila_detalle("system-users-symbolic", "Intérprete");
        let album = Self::construir_fila_detalle("media-optical-symbolic", "Álbum");
        detalles.attach(&titulo.0, 0, 0, 1, 1);
        detalles.attach(&Separator::new(Orientation::Horizontal), 0, 1, 1, 1);
        detalles.attach(&performer.0, 0, 2, 1, 1);
        detalles.attach(&Separator::new(Orientation::Horizontal), 0, 3, 1, 1);
        detalles.attach(&album.0, 0, 4, 1, 1);
        panel.append(&detalles);

        let mensaje = Label::new(Some("Selecciona una rola"));
        mensaje.set_wrap(true);
        panel.append(&mensaje);

        (
            panel,
            PanelSeleccion {
                caratula,
                album: album.1,
                performer: performer.1,
                titulo: titulo.1,
                mensaje,
            },
        )
    }

    fn construir_fila_detalle(icono: &str, texto: &str) -> (GtkBox, Label) {
        let fila = GtkBox::new(Orientation::Horizontal, 12);
        fila.set_margin_start(12);
        fila.set_margin_end(12);
        fila.set_margin_top(8);
        fila.set_margin_bottom(8);
        let imagen = Image::from_icon_name(icono);
        let etiqueta = Label::new(Some(texto));
        etiqueta.set_xalign(0.0);
        etiqueta.set_hexpand(true);
        etiqueta.set_ellipsize(gtk::pango::EllipsizeMode::End);
        fila.append(&imagen);
        fila.append(&etiqueta);
        (fila, etiqueta)
    }

    fn construir_panel_derecho() -> (GtkBox, Stack, Grid, FlowBox, Label) {
        let panel = GtkBox::new(Orientation::Vertical, 0);
        panel.set_hexpand(true);
        panel.set_vexpand(true);
        panel.set_margin_start(16);
        panel.set_margin_end(16);
        panel.set_margin_top(12);
        panel.set_margin_bottom(12);
        let (tabla_desplazable, tabla) = Self::construir_tabla_canciones();
        let vistas = Stack::new();
        vistas.add_named(&tabla_desplazable, Some("tabla"));
        let (recuadros_desplazables, recuadros) = Self::construir_vista_recuadros();
        vistas.add_named(&recuadros_desplazables, Some("recuadros"));
        vistas.set_hexpand(true);
        vistas.set_vexpand(true);
        vistas.set_visible_child_name("tabla");
        panel.append(&vistas);

        let mensaje = Label::new(Some(""));
        panel.append(&mensaje);
        (panel, vistas, tabla, recuadros, mensaje)
    }

    fn construir_tabla_canciones() -> (ScrolledWindow, Grid) {
        let tabla = Grid::new();
        tabla.set_column_spacing(16);
        tabla.set_row_spacing(0);
        tabla.add_css_class("view");

        let desplazamiento = ScrolledWindow::builder()
            .child(&tabla)
            .vexpand(true)
            .hexpand(true)
            .build();
        (desplazamiento, tabla)
    }

    fn construir_vista_recuadros() -> (ScrolledWindow, FlowBox) {
        let recuadros = FlowBox::new();
        recuadros.set_selection_mode(gtk::SelectionMode::None);
        recuadros.set_homogeneous(true);
        recuadros.set_min_children_per_line(1);
        recuadros.set_max_children_per_line(4);
        recuadros.set_row_spacing(16);
        recuadros.set_column_spacing(16);
        recuadros.set_margin_top(12);
        recuadros.set_margin_bottom(12);
        recuadros.add_css_class("view");
        recuadros.set_valign(gtk::Align::Start);

        let desplazamiento = ScrolledWindow::builder()
            .child(&recuadros)
            .vexpand(true)
            .hexpand(true)
            .build();
        (desplazamiento, recuadros)
    }
}

fn actualizar_tabla(tabla: &Grid, rolas: &[Rolita], resultados: &PanelResultados) {
    while let Some(hijo) = tabla.first_child() {
        tabla.remove(&hijo);
    }

    for (columna, titulo) in ["Title", "Performer", "Album", "Genre"].iter().enumerate() {
        agregar_celda(tabla, titulo, columna as i32, 0, true);
    }

    let separador = Separator::new(Orientation::Horizontal);
    tabla.attach(&separador, 0, 1, 4, 1);

    let seleccionada = resultados.ruta_seleccionada.borrow().clone();
    for (indice, rola) in rolas.iter().enumerate() {
        let fila = (indice as i32) * 2 + 2;
        let contenido = Grid::new();
        contenido.set_column_spacing(16);
        contenido.set_hexpand(true);
        agregar_celda(&contenido, &rola.titulo, 0, 0, false);
        agregar_celda(&contenido, &rola.performer, 1, 0, false);
        agregar_celda(&contenido, &rola.album, 2, 0, false);
        agregar_celda(
            &contenido,
            rola.genero.as_deref().unwrap_or("—"),
            3,
            0,
            false,
        );
        let boton = Button::new();
        boton.set_child(Some(&contenido));
        boton.set_hexpand(true);
        boton.add_css_class("flat");
        if seleccionada.as_ref() == Some(&rola.path) {
            boton.add_css_class("selected");
            *resultados.boton_seleccionado.borrow_mut() = Some(boton.clone());
        }
        let resultados_boton = resultados.clone();
        let rola_boton = rola.clone();
        boton.connect_clicked(move |boton| resultados_boton.seleccionar(&rola_boton, boton));
        tabla.attach(&boton, 0, fila, 4, 1);

        if indice + 1 < rolas.len() {
            let separador = Separator::new(Orientation::Horizontal);
            tabla.attach(&separador, 0, fila + 1, 4, 1);
        }
    }
}

fn actualizar_recuadros(recuadros: &FlowBox, rolas: &[Rolita], resultados: &PanelResultados) {
    while let Some(hijo) = recuadros.first_child() {
        recuadros.remove(&hijo);
    }

    let seleccionada = resultados.ruta_seleccionada.borrow().clone();
    for rola in rolas {
        let recuadro = GtkBox::new(Orientation::Vertical, 6);
        recuadro.set_size_request(190, -1);
        recuadro.set_margin_start(8);
        recuadro.set_margin_end(8);
        recuadro.set_margin_top(8);
        recuadro.set_margin_bottom(8);
        recuadro.add_css_class("card");

        agregar_caratula(&recuadro, rola);
        let album = Label::new(Some(&rola.album));
        album.set_ellipsize(gtk::pango::EllipsizeMode::End);
        album.set_max_width_chars(24);
        album.add_css_class("heading");
        let titulo = Label::new(Some(&rola.titulo));
        titulo.set_ellipsize(gtk::pango::EllipsizeMode::End);
        titulo.set_max_width_chars(24);
        let performer = Label::new(Some(&rola.performer));
        performer.set_ellipsize(gtk::pango::EllipsizeMode::End);
        performer.set_max_width_chars(24);
        performer.add_css_class("dim-label");
        recuadro.append(&album);
        recuadro.append(&titulo);
        recuadro.append(&performer);
        let boton = Button::new();
        boton.set_child(Some(&recuadro));
        boton.add_css_class("flat");
        if seleccionada.as_ref() == Some(&rola.path) {
            boton.add_css_class("selected");
            *resultados.boton_seleccionado.borrow_mut() = Some(boton.clone());
        }
        let resultados_boton = resultados.clone();
        let rola_boton = rola.clone();
        boton.connect_clicked(move |boton| resultados_boton.seleccionar(&rola_boton, boton));
        recuadros.append(&boton);
    }
}

fn agregar_caratula(recuadro: &GtkBox, rola: &Rolita) {
    let imagen = Image::new();
    imagen.set_pixel_size(180);
    imagen.set_size_request(180, 180);

    let textura = rola.caratula.as_ref().and_then(|bytes| {
        let bytes = gtk::glib::Bytes::from_owned(bytes.clone());
        gtk::gdk::Texture::from_bytes(&bytes).ok()
    });
    if let Some(textura) = textura {
        imagen.set_paintable(Some(&textura));
    } else {
        imagen.set_icon_name(Some("audio-x-generic-symbolic"));
    }

    recuadro.append(&imagen);
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
