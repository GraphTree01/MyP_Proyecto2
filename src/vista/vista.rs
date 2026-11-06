use gtk::prelude::*;
use gtk::{
    self, Application, ApplicationWindow, Box as GtkBox, Button, HeaderBar, Label, Orientation,
    Separator,
};

const APP_ID: &str = "com.proyecto_dos.myp";

/// Representa la vista principal de la aplicación.
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

        let derecha = GtkBox::new(Orientation::Vertical, 10);
        let texto_derecha = Label::new(Some("Contenido original"));
        derecha.append(&texto_derecha);

        let separador = Separator::new(Orientation::Vertical);

        contenido.append(&izquierda);
        contenido.append(&separador);
        contenido.append(&derecha);

        button.connect_clicked(move |_| {
            texto_derecha.set_text("¡Contenido actualizado!");
        });

        let window = ApplicationWindow::builder()
            .application(application)
            .title("Mi aplicación")
            .default_width(500)
            .default_height(350)
            .build();

        window.set_titlebar(Some(&header));
        window.set_child(Some(&contenido));
        window.present();
    }
}
