use crate::modelo::metadatos::Metadatos;
use crate::modelo::operaciones::Operaciones;
use crate::modelo::rola::Rolita;
use gtk::prelude::*;
use gtk::{
    self, Application, ApplicationWindow, Box as GtkBox, Button, Entry, FileDialog, FlowBox, Frame,
    Grid, HeaderBar, Image, Label, Orientation, ScrolledWindow, Separator, Stack, ToggleButton,
    Window,
};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

const APP_ID: &str = "com.proyecto_dos.myp";
type BusquedaCallback = Rc<dyn Fn(String, bool) -> Result<Vec<Rolita>, String>>;

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
    rola_seleccionada: Rc<RefCell<Option<Rolita>>>,
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
                    *self.rola_seleccionada.borrow_mut() = Some(rola.clone());
                    self.detalle.mostrar(rola);
                } else if seleccion.is_some() {
                    *self.ruta_seleccionada.borrow_mut() = None;
                    *self.rola_seleccionada.borrow_mut() = None;
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
        *self.rola_seleccionada.borrow_mut() = Some(rola.clone());
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
    pub fn iniciar<F, B>(
        &self,
        rolas: Vec<Rolita>,
        operaciones: Rc<Operaciones>,
        al_minar: F,
        al_buscar: B,
    ) where
        F: Fn(PathBuf) -> Result<(), String> + Clone + 'static,
        B: Fn(String, bool) -> Result<Vec<Rolita>, String> + Clone + 'static,
    {
        self.aplicacion.connect_activate(move |application| {
            Self::construir_ui(
                application,
                rolas.clone(),
                Rc::clone(&operaciones),
                al_minar.clone(),
                al_buscar.clone(),
            )
        });
        self.aplicacion.run();
    }

    fn construir_ui<F, B>(
        application: &Application,
        rolas: Vec<Rolita>,
        operaciones: Rc<Operaciones>,
        al_minar: F,
        al_buscar: B,
    ) where
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
            rola_seleccionada: Rc::new(RefCell::new(None)),
            boton_seleccionado: Rc::new(RefCell::new(None)),
        };
        resultados.actualizar_vista(&rolas);
        let al_buscar: Rc<dyn Fn(String, bool) -> Result<Vec<Rolita>, String>> = Rc::new(al_buscar);
        let al_minar: Rc<dyn Fn(PathBuf) -> Result<(), String>> = Rc::new(al_minar);
        header.pack_start(&Self::construir_botones_edicion(
            &window,
            resultados.clone(),
            Rc::clone(&operaciones),
            Rc::clone(&al_buscar),
        ));
        header.pack_end(&Self::construir_botones_encabezado(
            &window, al_minar, al_buscar, resultados,
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

    fn construir_botones_edicion(
        window: &ApplicationWindow,
        resultados: PanelResultados,
        operaciones: Rc<Operaciones>,
        al_buscar: Rc<dyn Fn(String, bool) -> Result<Vec<Rolita>, String>>,
    ) -> GtkBox {
        let edicion = GtkBox::new(Orientation::Horizontal, 0);
        edicion.add_css_class("linked");
        let boton_mas = Button::from_icon_name("list-add-symbolic");
        boton_mas.set_tooltip_text(Some("Agregar persona o miembro"));
        let boton_performer = Button::from_icon_name("system-users-symbolic");
        boton_performer.set_tooltip_text(Some("Editar intérprete"));
        let boton_album = Button::from_icon_name("media-optical-symbolic");
        boton_album.set_tooltip_text(Some("Editar álbum"));
        let boton_titulo = Button::from_icon_name("audio-x-generic-symbolic");
        boton_titulo.set_tooltip_text(Some("Editar canción"));
        edicion.append(&boton_mas);
        edicion.append(&boton_performer);
        edicion.append(&boton_album);
        edicion.append(&boton_titulo);

        let padre = window.clone();
        let resultados_mas = resultados.clone();
        let operaciones_mas = Rc::clone(&operaciones);
        boton_mas.connect_clicked(move |_| {
            abrir_accion_agregar(&padre, &resultados_mas, &operaciones_mas);
        });

        let padre = window.clone();
        let resultados_performer = resultados.clone();
        let operaciones_performer = Rc::clone(&operaciones);
        let buscar_performer = Rc::clone(&al_buscar);
        boton_performer.connect_clicked(move |_| {
            abrir_formulario_performer(
                &padre,
                &resultados_performer,
                &operaciones_performer,
                &buscar_performer,
            );
        });

        let padre = window.clone();
        let resultados_album = resultados.clone();
        let operaciones_album = Rc::clone(&operaciones);
        let buscar_album = Rc::clone(&al_buscar);
        boton_album.connect_clicked(move |_| {
            abrir_formulario_album(&padre, &resultados_album, &operaciones_album, &buscar_album);
        });

        let padre = window.clone();
        let resultados_titulo = resultados;
        let operaciones_titulo = operaciones;
        let buscar_titulo = al_buscar;
        boton_titulo.connect_clicked(move |_| {
            abrir_formulario_rola(
                &padre,
                &resultados_titulo,
                &operaciones_titulo,
                &buscar_titulo,
            );
        });
        edicion
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

fn rola_actual(resultados: &PanelResultados) -> Option<Rolita> {
    let rola = resultados.rola_seleccionada.borrow().clone();
    if rola.is_none() {
        resultados
            .mensaje
            .set_text("Selecciona una canción antes de editar.");
    }
    rola
}

fn actualizar_resultados(
    resultados: &PanelResultados,
    buscar: &dyn Fn(String, bool) -> Result<Vec<Rolita>, String>,
) {
    let expresion = resultados.expresion_actual.borrow().clone();
    resultados.ejecutar_busqueda(expresion, buscar);
}

fn abrir_formulario_album(
    padre: &ApplicationWindow,
    resultados: &PanelResultados,
    operaciones: &Rc<Operaciones>,
    buscar: &BusquedaCallback,
) {
    let Some(rola) = rola_actual(resultados) else {
        return;
    };
    let album = match operaciones.obtener_album(rola.id_album) {
        Ok(Some(album)) => album,
        Ok(None) => {
            resultados
                .mensaje
                .set_text("No se encontró el álbum seleccionado.");
            return;
        }
        Err(error) => {
            resultados
                .mensaje
                .set_text(&format!("No se pudo consultar el álbum: {error}"));
            return;
        }
    };

    let ventana = Window::builder()
        .title("Editar álbum")
        .transient_for(padre)
        .modal(true)
        .default_width(360)
        .build();
    let contenido = GtkBox::new(Orientation::Vertical, 8);
    contenido.set_margin_top(16);
    contenido.set_margin_bottom(16);
    contenido.set_margin_start(16);
    contenido.set_margin_end(16);
    contenido.append(&Label::new(Some("Nombre del álbum")));
    let nombre = Entry::new();
    nombre.set_text(&album.name);
    contenido.append(&nombre);
    contenido.append(&Label::new(Some("Año (opcional)")));
    let anio = Entry::new();
    if let Some(anio_album) = album.year {
        anio.set_text(&anio_album.to_string());
    }
    contenido.append(&anio);
    let mensaje = Label::new(Some(""));
    mensaje.set_wrap(true);
    contenido.append(&mensaje);
    let acciones = GtkBox::new(Orientation::Horizontal, 8);
    let guardar = Button::with_label("Guardar");
    let eliminar = Button::with_label("Eliminar álbum");
    acciones.append(&eliminar);
    acciones.append(&guardar);
    contenido.append(&acciones);
    ventana.set_child(Some(&contenido));

    let ventana_guardar = ventana.clone();
    let mensaje_guardar = mensaje.clone();
    let nombre_guardar = nombre.clone();
    let anio_guardar = anio.clone();
    let resultados_guardar = resultados.clone();
    let operaciones_guardar = Rc::clone(operaciones);
    let buscar_guardar = Rc::clone(buscar);
    let id_album = rola.id_album;
    guardar.connect_clicked(move |_| {
        let nombre = nombre_guardar.text().trim().to_owned();
        if nombre.is_empty() {
            mensaje_guardar.set_text("El nombre del álbum no puede estar vacío.");
            return;
        }
        let texto_anio = anio_guardar.text();
        let anio = if texto_anio.trim().is_empty() {
            None
        } else {
            match texto_anio.trim().parse::<i32>() {
                Ok(anio) => Some(anio),
                Err(_) => {
                    mensaje_guardar.set_text("El año debe ser un número entero.");
                    return;
                }
            }
        };
        match operaciones_guardar.actualizar_album(id_album, &nombre, anio) {
            Ok(true) => {
                actualizar_resultados(&resultados_guardar, &*buscar_guardar);
                ventana_guardar.close();
            }
            Ok(false) => mensaje_guardar.set_text("No se encontró el álbum para actualizar."),
            Err(error) => mensaje_guardar.set_text(&error.to_string()),
        }
    });

    let padre_eliminar = ventana.clone();
    let ventana_eliminar = ventana.clone();
    let mensaje_eliminar = mensaje.clone();
    let resultados_eliminar = resultados.clone();
    let operaciones_eliminar = Rc::clone(operaciones);
    let buscar_eliminar = Rc::clone(buscar);
    eliminar.connect_clicked(move |_| {
        let resultados = resultados_eliminar.clone();
        let operaciones = Rc::clone(&operaciones_eliminar);
        let buscar = Rc::clone(&buscar_eliminar);
        let ventana = ventana_eliminar.clone();
        let mensaje = mensaje_eliminar.clone();
        let mensaje_error = mensaje_eliminar.clone();
        confirmar_eliminacion(
            &padre_eliminar,
            "¿Eliminar este álbum?",
            move || match operaciones.eliminar_album(id_album) {
                Ok(true) => {
                    actualizar_resultados(&resultados, &*buscar);
                    ventana.close();
                }
                Ok(false) => mensaje.set_text("No se encontró el álbum para eliminar."),
                Err(error) => mensaje.set_text(&error.to_string()),
            },
            move |error| mensaje_error.set_text(&error),
        );
    });
    ventana.present();
}

fn abrir_formulario_rola(
    padre: &ApplicationWindow,
    resultados: &PanelResultados,
    operaciones: &Rc<Operaciones>,
    buscar: &BusquedaCallback,
) {
    let Some(rola) = rola_actual(resultados) else {
        return;
    };
    let ventana = Window::builder()
        .title("Editar canción")
        .transient_for(padre)
        .modal(true)
        .default_width(360)
        .build();
    let contenido = GtkBox::new(Orientation::Vertical, 8);
    contenido.set_margin_top(16);
    contenido.set_margin_bottom(16);
    contenido.set_margin_start(16);
    contenido.set_margin_end(16);
    contenido.append(&Label::new(Some("Título")));
    let titulo = Entry::new();
    titulo.set_text(&rola.titulo);
    contenido.append(&titulo);
    let mensaje = Label::new(Some(""));
    mensaje.set_wrap(true);
    contenido.append(&mensaje);
    let acciones = GtkBox::new(Orientation::Horizontal, 8);
    let eliminar = Button::with_label("Eliminar canción");
    let guardar = Button::with_label("Guardar");
    acciones.append(&eliminar);
    acciones.append(&guardar);
    contenido.append(&acciones);
    ventana.set_child(Some(&contenido));

    let ventana_guardar = ventana.clone();
    let titulo_guardar = titulo.clone();
    let mensaje_guardar = mensaje.clone();
    let resultados_guardar = resultados.clone();
    let operaciones_guardar = Rc::clone(operaciones);
    let buscar_guardar = Rc::clone(buscar);
    guardar.connect_clicked(move |_| {
        let titulo = titulo_guardar.text().trim().to_owned();
        if titulo.is_empty() {
            mensaje_guardar.set_text("El título no puede estar vacío.");
            return;
        }
        match operaciones_guardar.actualizar_rola(
            rola.id_rola,
            &titulo,
            rola.id_performer,
            rola.id_album,
        ) {
            Ok(true) => {
                actualizar_resultados(&resultados_guardar, &*buscar_guardar);
                ventana_guardar.close();
            }
            Ok(false) => mensaje_guardar.set_text("No se encontró la canción para actualizar."),
            Err(error) => mensaje_guardar.set_text(&error.to_string()),
        }
    });

    let padre_eliminar = ventana.clone();
    let ventana_eliminar = ventana.clone();
    let mensaje_eliminar = mensaje.clone();
    let resultados_eliminar = resultados.clone();
    let operaciones_eliminar = Rc::clone(operaciones);
    let buscar_eliminar = Rc::clone(buscar);
    eliminar.connect_clicked(move |_| {
        let resultados = resultados_eliminar.clone();
        let operaciones = Rc::clone(&operaciones_eliminar);
        let buscar = Rc::clone(&buscar_eliminar);
        let ventana = ventana_eliminar.clone();
        let mensaje = mensaje_eliminar.clone();
        let mensaje_error = mensaje_eliminar.clone();
        confirmar_eliminacion(
            &padre_eliminar,
            "¿Eliminar esta canción?",
            move || match operaciones.eliminar_rola(rola.id_rola) {
                Ok(true) => {
                    actualizar_resultados(&resultados, &*buscar);
                    ventana.close();
                }
                Ok(false) => mensaje.set_text("No se encontró la canción para eliminar."),
                Err(error) => mensaje.set_text(&error.to_string()),
            },
            move |error| mensaje_error.set_text(&error),
        );
    });
    ventana.present();
}

fn abrir_formulario_performer(
    padre: &ApplicationWindow,
    resultados: &PanelResultados,
    operaciones: &Rc<Operaciones>,
    buscar: &BusquedaCallback,
) {
    let Some(rola) = rola_actual(resultados) else {
        return;
    };
    let performer = match operaciones.obtener_performer(rola.id_performer) {
        Ok(Some(performer)) => performer,
        Ok(None) => {
            resultados
                .mensaje
                .set_text("No se encontró el intérprete seleccionado.");
            return;
        }
        Err(error) => {
            resultados
                .mensaje
                .set_text(&format!("No se pudo consultar el intérprete: {error}"));
            return;
        }
    };
    let persona = match operaciones.obtener_persona(rola.id_performer) {
        Ok(persona) => persona,
        Err(error) => {
            resultados.mensaje.set_text(&format!(
                "No se pudieron consultar los datos personales: {error}"
            ));
            return;
        }
    };
    let grupo = match operaciones.obtener_grupo(rola.id_performer) {
        Ok(grupo) => grupo,
        Err(error) => {
            resultados.mensaje.set_text(&format!(
                "No se pudieron consultar los datos del grupo: {error}"
            ));
            return;
        }
    };

    let ventana = Window::builder()
        .title("Editar intérprete")
        .transient_for(padre)
        .modal(true)
        .default_width(420)
        .build();
    let contenido = GtkBox::new(Orientation::Vertical, 8);
    contenido.set_margin_top(16);
    contenido.set_margin_bottom(16);
    contenido.set_margin_start(16);
    contenido.set_margin_end(16);
    contenido.append(&Label::new(Some("Tipo de intérprete")));
    let tipo = gtk::DropDown::from_strings(&["Persona", "Grupo", "Desconocido"]);
    tipo.set_selected(performer.id_type.id() as u32);
    contenido.append(&tipo);
    contenido.append(&Label::new(Some("Nombre mostrado")));
    let nombre = Entry::new();
    nombre.set_text(&performer.name);
    contenido.append(&nombre);

    let datos_persona = GtkBox::new(Orientation::Vertical, 6);
    datos_persona.append(&Label::new(Some("Nombre artístico")));
    let nombre_artistico = Entry::new();
    if let Some(persona) = &persona
        && let Some(valor) = &persona.stage_name
    {
        nombre_artistico.set_text(valor);
    } else {
        nombre_artistico.set_text(&performer.name);
    }
    datos_persona.append(&nombre_artistico);
    datos_persona.append(&Label::new(Some("Nombre real")));
    let nombre_real = Entry::new();
    if let Some(persona) = &persona
        && let Some(valor) = &persona.real_name
    {
        nombre_real.set_text(valor);
    }
    datos_persona.append(&nombre_real);
    datos_persona.append(&Label::new(Some("Nacimiento (AAAA-MM-DD)")));
    let nacimiento = Entry::new();
    if let Some(persona) = &persona
        && let Some(valor) = &persona.birth_date
    {
        nacimiento.set_text(valor);
    }
    datos_persona.append(&nacimiento);
    datos_persona.append(&Label::new(Some("Fallecimiento (AAAA-MM-DD)")));
    let fallecimiento = Entry::new();
    if let Some(persona) = &persona
        && let Some(valor) = &persona.death_date
    {
        fallecimiento.set_text(valor);
    }
    datos_persona.append(&fallecimiento);
    contenido.append(&datos_persona);

    let datos_grupo = GtkBox::new(Orientation::Vertical, 6);
    datos_grupo.append(&Label::new(Some("Inicio (AAAA-MM-DD)")));
    let inicio_grupo = Entry::new();
    if let Some(grupo) = &grupo
        && let Some(valor) = &grupo.start_date
    {
        inicio_grupo.set_text(valor);
    }
    datos_grupo.append(&inicio_grupo);
    datos_grupo.append(&Label::new(Some("Finalización (AAAA-MM-DD)")));
    let fin_grupo = Entry::new();
    if let Some(grupo) = &grupo
        && let Some(valor) = &grupo.end_date
    {
        fin_grupo.set_text(valor);
    }
    datos_grupo.append(&fin_grupo);
    contenido.append(&datos_grupo);
    let actualizar_secciones = {
        let datos_persona = datos_persona.clone();
        let datos_grupo = datos_grupo.clone();
        let tipo = tipo.clone();
        move || {
            datos_persona.set_visible(tipo.selected() == 0);
            datos_grupo.set_visible(tipo.selected() == 1);
        }
    };
    actualizar_secciones();
    tipo.connect_selected_notify(move |_| actualizar_secciones());

    let mensaje = Label::new(Some(""));
    mensaje.set_wrap(true);
    contenido.append(&mensaje);
    let acciones = GtkBox::new(Orientation::Horizontal, 8);
    let eliminar = Button::with_label("Eliminar intérprete");
    let miembros = Button::with_label("Ver miembros");
    let guardar = Button::with_label("Guardar");
    miembros.set_visible(performer.id_type == crate::modelo::r#type::Type::Group);
    acciones.append(&eliminar);
    acciones.append(&miembros);
    acciones.append(&guardar);
    contenido.append(&acciones);
    ventana.set_child(Some(&contenido));

    let padre_miembros = ventana.clone();
    let operaciones_miembros = Rc::clone(operaciones);
    let mensaje_miembros = mensaje.clone();
    miembros.connect_clicked(move |_| {
        match operaciones_miembros.obtener_performer(rola.id_performer) {
            Ok(Some(performer)) if performer.id_type == crate::modelo::r#type::Type::Group => {
                abrir_ventana_miembros(&padre_miembros, rola.id_performer, &operaciones_miembros);
            }
            Ok(_) => mensaje_miembros.set_text("El intérprete ya no es un grupo."),
            Err(error) => mensaje_miembros.set_text(&error.to_string()),
        }
    });

    let ventana_guardar = ventana.clone();
    let tipo_guardar = tipo.clone();
    let nombre_guardar = nombre.clone();
    let nombre_artistico_guardar = nombre_artistico.clone();
    let nombre_real_guardar = nombre_real.clone();
    let nacimiento_guardar = nacimiento.clone();
    let fallecimiento_guardar = fallecimiento.clone();
    let inicio_grupo_guardar = inicio_grupo.clone();
    let fin_grupo_guardar = fin_grupo.clone();
    let mensaje_guardar = mensaje.clone();
    let operaciones_guardar = Rc::clone(operaciones);
    let resultados_guardar = resultados.clone();
    let buscar_guardar = Rc::clone(buscar);
    guardar.connect_clicked(move |_| {
        let nombre = nombre_guardar.text().trim().to_owned();
        if nombre.is_empty() {
            mensaje_guardar.set_text("El nombre del intérprete no puede estar vacío.");
            return;
        }
        let respuesta = match tipo_guardar.selected() {
            0 => operaciones_guardar.guardar_performer_persona(
                rola.id_performer,
                &nombre,
                &crate::modelo::person::Person {
                    id_person: Some(rola.id_performer),
                    stage_name: texto_opcional(&nombre_artistico_guardar),
                    real_name: texto_opcional(&nombre_real_guardar),
                    birth_date: texto_opcional(&nacimiento_guardar),
                    death_date: texto_opcional(&fallecimiento_guardar),
                },
            ),
            1 => operaciones_guardar.guardar_performer_grupo(
                rola.id_performer,
                &nombre,
                &crate::modelo::group::Group {
                    id_group: Some(rola.id_performer),
                    name: nombre.clone(),
                    start_date: texto_opcional(&inicio_grupo_guardar),
                    end_date: texto_opcional(&fin_grupo_guardar),
                },
            ),
            _ => operaciones_guardar
                .guardar_performer_desconocido(rola.id_performer, &nombre)
                .and_then(|actualizado| {
                    if actualizado {
                        Ok(())
                    } else {
                        Err(crate::modelo::error::ErrorModelo::Validacion(
                            "No se encontró el intérprete para actualizar.".to_owned(),
                        ))
                    }
                }),
        };
        match respuesta {
            Ok(()) => {
                actualizar_resultados(&resultados_guardar, &*buscar_guardar);
                ventana_guardar.close();
            }
            Err(error) => mensaje_guardar.set_text(&error.to_string()),
        }
    });

    let padre_eliminar = ventana.clone();
    let ventana_eliminar = ventana.clone();
    let mensaje_eliminar = mensaje.clone();
    let operaciones_eliminar = Rc::clone(operaciones);
    let resultados_eliminar = resultados.clone();
    let buscar_eliminar = Rc::clone(buscar);
    eliminar.connect_clicked(move |_| {
        let operaciones = Rc::clone(&operaciones_eliminar);
        let resultados = resultados_eliminar.clone();
        let buscar = Rc::clone(&buscar_eliminar);
        let ventana = ventana_eliminar.clone();
        let mensaje = mensaje_eliminar.clone();
        let mensaje_error = mensaje_eliminar.clone();
        confirmar_eliminacion(
            &padre_eliminar,
            "¿Eliminar este intérprete?",
            move || match operaciones.eliminar_performer(rola.id_performer) {
                Ok(true) => {
                    actualizar_resultados(&resultados, &*buscar);
                    ventana.close();
                }
                Ok(false) => mensaje.set_text("No se encontró el intérprete para eliminar."),
                Err(error) => mensaje.set_text(&error.to_string()),
            },
            move |error| mensaje_error.set_text(&error),
        );
    });
    ventana.present();
}

fn texto_opcional(entrada: &Entry) -> Option<String> {
    let texto = entrada.text();
    let texto = texto.trim();
    (!texto.is_empty()).then(|| texto.to_owned())
}

fn abrir_accion_agregar(
    padre: &ApplicationWindow,
    resultados: &PanelResultados,
    operaciones: &Rc<Operaciones>,
) {
    if let Some(rola) = rola_actual(resultados) {
        match operaciones.obtener_performer(rola.id_performer) {
            Ok(Some(performer)) if performer.id_type == crate::modelo::r#type::Type::Group => {
                abrir_ventana_miembros(padre.upcast_ref(), rola.id_performer, operaciones);
                return;
            }
            Ok(_) => {}
            Err(error) => {
                resultados
                    .mensaje
                    .set_text(&format!("No se pudo consultar el intérprete: {error}"));
                return;
            }
        }
    }
    abrir_formulario_persona(
        padre.upcast_ref(),
        operaciones,
        Some(resultados.clone()),
        None,
        None,
    );
}

fn abrir_formulario_persona(
    padre: &Window,
    operaciones: &Rc<Operaciones>,
    resultados: Option<PanelResultados>,
    grupo: Option<i64>,
    al_guardar: Option<Rc<dyn Fn()>>,
) {
    let ventana = Window::builder()
        .title(if grupo.is_some() {
            "Agregar miembro"
        } else {
            "Agregar persona"
        })
        .transient_for(padre)
        .modal(true)
        .default_width(380)
        .build();
    let contenido = GtkBox::new(Orientation::Vertical, 8);
    contenido.set_margin_top(16);
    contenido.set_margin_bottom(16);
    contenido.set_margin_start(16);
    contenido.set_margin_end(16);
    let artistico = Entry::new();
    artistico.set_placeholder_text(Some("Nombre artístico (opcional)"));
    contenido.append(&artistico);
    let real = Entry::new();
    real.set_placeholder_text(Some("Nombre real (opcional)"));
    contenido.append(&real);
    let nacimiento = Entry::new();
    nacimiento.set_placeholder_text(Some("Nacimiento: AAAA-MM-DD (opcional)"));
    contenido.append(&nacimiento);
    let fallecimiento = Entry::new();
    fallecimiento.set_placeholder_text(Some("Fallecimiento: AAAA-MM-DD (opcional)"));
    contenido.append(&fallecimiento);
    let mensaje = Label::new(Some(""));
    mensaje.set_wrap(true);
    contenido.append(&mensaje);
    let guardar = Button::with_label(if grupo.is_some() {
        "Agregar a la banda"
    } else {
        "Guardar persona"
    });
    contenido.append(&guardar);
    ventana.set_child(Some(&contenido));

    let ventana_guardar = ventana.clone();
    let artistico_guardar = artistico.clone();
    let real_guardar = real.clone();
    let nacimiento_guardar = nacimiento.clone();
    let fallecimiento_guardar = fallecimiento.clone();
    let mensaje_guardar = mensaje.clone();
    let operaciones_guardar = Rc::clone(operaciones);
    let resultados_guardar = resultados;
    let al_guardar_guardar = al_guardar;
    let grupo_guardar = grupo;
    guardar.connect_clicked(move |_| {
        let stage_name = texto_opcional(&artistico_guardar);
        let real_name = texto_opcional(&real_guardar);
        if stage_name.is_none() && real_name.is_none() {
            mensaje_guardar.set_text("Ingresa al menos un nombre para la persona.");
            return;
        }
        let persona = crate::modelo::person::Person {
            id_person: None,
            stage_name,
            real_name,
            birth_date: texto_opcional(&nacimiento_guardar),
            death_date: texto_opcional(&fallecimiento_guardar),
        };
        match operaciones_guardar.crear_persona(&persona, grupo_guardar) {
            Ok(_) => {
                if grupo_guardar.is_some()
                    && let Some(resultados) = &resultados_guardar
                {
                    resultados
                        .mensaje
                        .set_text("La persona se agregó al grupo.");
                }
                if let Some(al_guardar) = &al_guardar_guardar {
                    al_guardar();
                }
                ventana_guardar.close();
            }
            Err(error) => mensaje_guardar.set_text(&error.to_string()),
        }
    });
    ventana.present();
}

fn abrir_ventana_miembros(padre: &Window, id_group: i64, operaciones: &Rc<Operaciones>) {
    let ventana = Window::builder()
        .title("Miembros del grupo")
        .transient_for(padre)
        .modal(true)
        .default_width(560)
        .default_height(420)
        .build();
    let contenido = GtkBox::new(Orientation::Vertical, 8);
    contenido.set_margin_top(12);
    contenido.set_margin_bottom(12);
    contenido.set_margin_start(12);
    contenido.set_margin_end(12);
    ventana.set_child(Some(&contenido));
    actualizar_ventana_miembros(&contenido, id_group, operaciones);
    ventana.present();
}

fn actualizar_ventana_miembros(contenido: &GtkBox, id_group: i64, operaciones: &Rc<Operaciones>) {
    while let Some(hijo) = contenido.first_child() {
        contenido.remove(&hijo);
    }
    let tabla = Grid::new();
    tabla.set_column_spacing(12);
    tabla.set_row_spacing(8);
    for (columna, titulo) in [
        "Nombre artístico",
        "Nombre real",
        "Nacimiento",
        "Fallecimiento",
        "Acción",
    ]
    .iter()
    .enumerate()
    {
        let etiqueta = Label::new(Some(titulo));
        etiqueta.add_css_class("heading");
        tabla.attach(&etiqueta, columna as i32, 0, 1, 1);
    }
    let personas = match operaciones.listar_miembros(id_group) {
        Ok(personas) => personas,
        Err(error) => {
            contenido.append(&Label::new(Some(&format!(
                "No se pudieron cargar los miembros: {error}"
            ))));
            return;
        }
    };
    for (indice, persona) in personas.iter().enumerate() {
        let fila = indice as i32 + 1;
        tabla.attach(
            &Label::new(Some(persona.stage_name.as_deref().unwrap_or("—"))),
            0,
            fila,
            1,
            1,
        );
        tabla.attach(
            &Label::new(Some(persona.real_name.as_deref().unwrap_or("—"))),
            1,
            fila,
            1,
            1,
        );
        tabla.attach(
            &Label::new(Some(persona.birth_date.as_deref().unwrap_or("—"))),
            2,
            fila,
            1,
            1,
        );
        tabla.attach(
            &Label::new(Some(persona.death_date.as_deref().unwrap_or("—"))),
            3,
            fila,
            1,
            1,
        );
        let quitar = Button::with_label("Quitar");
        if let Some(id_person) = persona.id_person {
            let contenido_refresh = contenido.clone();
            let operaciones_refresh = Rc::clone(operaciones);
            quitar.connect_clicked(move |_| {
                match operaciones_refresh.quitar_miembro(id_person, id_group) {
                    Ok(_) => actualizar_ventana_miembros(
                        &contenido_refresh,
                        id_group,
                        &operaciones_refresh,
                    ),
                    Err(error) => {
                        let error = Label::new(Some(&error.to_string()));
                        contenido_refresh.append(&error);
                    }
                }
            });
        }
        tabla.attach(&quitar, 4, fila, 1, 1);
    }
    let desplazamiento = ScrolledWindow::builder()
        .child(&tabla)
        .vexpand(true)
        .hexpand(true)
        .build();
    contenido.append(&desplazamiento);

    let personas_existentes = match operaciones.listar_personas() {
        Ok(personas) => personas,
        Err(error) => {
            contenido.append(&Label::new(Some(&format!(
                "No se pudo consultar el registro de personas: {error}"
            ))));
            return;
        }
    };
    let integrantes_actuales = personas
        .iter()
        .filter_map(|persona| persona.id_person)
        .collect::<std::collections::HashSet<_>>();
    let candidatas = personas_existentes
        .into_iter()
        .filter(|persona| {
            persona
                .id_person
                .is_some_and(|id| !integrantes_actuales.contains(&id))
        })
        .collect::<Vec<_>>();
    let fila_agregar = GtkBox::new(Orientation::Horizontal, 8);
    if !candidatas.is_empty() {
        let nombres = candidatas
            .iter()
            .map(|persona| {
                persona
                    .stage_name
                    .as_deref()
                    .or(persona.real_name.as_deref())
                    .unwrap_or("Persona sin nombre")
            })
            .collect::<Vec<_>>();
        let seleccion = gtk::DropDown::from_strings(&nombres);
        let seleccion_agregar = seleccion.clone();
        let agregar = Button::with_label("Agregar persona");
        let candidatas = Rc::new(candidatas);
        let operaciones_agregar = Rc::clone(operaciones);
        let contenido_agregar = contenido.clone();
        agregar.connect_clicked(move |_| {
            let seleccionado = seleccion_agregar.selected() as usize;
            let Some(persona) = candidatas.get(seleccionado) else {
                return;
            };
            if let Some(id_person) = persona.id_person {
                match operaciones_agregar.agregar_miembro(id_person, id_group) {
                    Ok(()) => actualizar_ventana_miembros(
                        &contenido_agregar,
                        id_group,
                        &operaciones_agregar,
                    ),
                    Err(error) => contenido_agregar.append(&Label::new(Some(&error.to_string()))),
                }
            }
        });
        fila_agregar.append(&seleccion);
        fila_agregar.append(&agregar);
    }
    let nuevo = Button::with_label("Crear persona y agregarla");
    let operaciones_nuevo = Rc::clone(operaciones);
    let contenido_nuevo = contenido.clone();
    nuevo.connect_clicked(move |_| {
        if let Some(padre) = contenido_nuevo.root().and_downcast::<Window>() {
            let contenido_refresh = contenido_nuevo.clone();
            let operaciones_refresh = Rc::clone(&operaciones_nuevo);
            let al_guardar: Rc<dyn Fn()> = Rc::new(move || {
                actualizar_ventana_miembros(&contenido_refresh, id_group, &operaciones_refresh);
            });
            abrir_formulario_persona(
                &padre,
                &operaciones_nuevo,
                None,
                Some(id_group),
                Some(al_guardar),
            );
        } else {
            contenido_nuevo.append(&Label::new(Some(
                "No se pudo abrir la ventana para crear un miembro.",
            )));
        }
    });
    fila_agregar.append(&nuevo);
    contenido.append(&fila_agregar);
}

fn confirmar_eliminacion(
    padre: &impl gtk::prelude::IsA<Window>,
    texto: &str,
    al_confirmar: impl FnOnce() + 'static,
    al_error: impl Fn(String) + 'static,
) {
    let dialogo = gtk::AlertDialog::builder()
        .message(texto)
        .buttons(["Cancelar", "Eliminar"])
        .default_button(0)
        .cancel_button(0)
        .build();
    dialogo.choose(
        Some(padre.upcast_ref()),
        None::<&gtk::gio::Cancellable>,
        move |respuesta| match respuesta {
            Ok(1) => al_confirmar(),
            Ok(_) => {}
            Err(error) => al_error(format!("No se pudo confirmar la acción: {error}")),
        },
    );
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
