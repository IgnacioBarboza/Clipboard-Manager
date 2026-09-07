use gtk4 as gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Label, Orientation, CssProvider};
use gtk::gdk::Display;
use gtk4_layer_shell::{Layer, LayerShell, KeyboardMode}; 
use std::time::{SystemTime, UNIX_EPOCH};
use circular_buffer::FixedCircularBuffer;


const APP_ID: &str = "gtk_popup";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    
    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);
    app.run();
}

fn load_css() {
    let provider = CssProvider::new();
    let css = "
        window { background-color: transparent; }
        .menu-recuadro { background-color: #121212; border-radius: 10px; }
        label { color: #FFFFFF; font-size: 16px; margin: 8px; font-weight: bold; }
    ";
    provider.load_from_data(css); 

    if let Some(display) = Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn build_ui(app: &Application) {
    let mut buffer = FixedCircularBuffer::<String, 5>::new();
    buffer.push_back("hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh".to_string());
    buffer.push_back("testing 1".to_string());
    buffer.push_back("testing 2".to_string());
    buffer.push_back("testing 3".to_string());
    buffer.push_back("testing 4".to_string());

    // 1. Contenedor exterior que ocupa toda la pantalla con fondo transparente
    let outer_box = Box::new(Orientation::Vertical, 0);
    outer_box.set_vexpand(true);
    outer_box.set_hexpand(true);
    outer_box.set_valign(gtk::Align::Center);
    outer_box.set_halign(gtk::Align::Center);

    // 2. Contenedor interior (el recuadro real que SÍ se puede clickear)
    let vbox = Box::new(Orientation::Vertical, 5);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);
    vbox.add_css_class("menu-recuadro");
    vbox.set_valign(gtk::Align::Center);
    vbox.set_halign(gtk::Align::Center);

    for texto in buffer.iter() {
        let label = Label::new(Some(texto));
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.set_xalign(0.0);
        vbox.append(&label);
    }

    outer_box.append(&vbox);

    let window = ApplicationWindow::builder()
        .application(app)
        .child(&outer_box)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Right, true);

    // Evento de teclado para cerrar con ESC
    let win_clone = window.clone();
    let key_controller = gtk::EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, keyval, _, _| {
        if keyval == gtk::gdk::Key::Escape {
            win_clone.close();
            true.into()
        } else {
            false.into()
        }
    });
    window.add_controller(key_controller);

    // Clic en la capa de afuera (outer_box) para cerrar
    let win_clone2 = window.clone();
    let click_outer = gtk::GestureClick::new();
    click_outer.connect_pressed(move |_, _, _, _| {
        win_clone2.close();
    });
    outer_box.add_controller(click_outer);

    // Evitar que los clics en el recuadro interior se propaguen al exterior
    let click_vbox = gtk::GestureClick::new();
    click_vbox.connect_pressed(move |controller, _, _, _| {
        controller.set_state(gtk::EventSequenceState::Claimed);
    });
    vbox.add_controller(click_vbox);

    window.present();
}