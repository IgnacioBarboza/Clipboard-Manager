use iced::event::{self, Event};
use iced::keyboard::{self, Key};
use iced::widget::{container, mouse_area, text};
use iced::{window, Color, Element, Length, Subscription, Task};
use iced_layershell::actions::LayerShellCustomActionWithId;
use iced_layershell::application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings};

#[derive(Default)]
struct AppState {}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(Event),
    CerrarMenu,
    IgnorarClic, // Usado como "escudo" para el recuadro central
}

impl TryFrom<Message> for LayerShellCustomActionWithId {
    type Error = Message;
    fn try_from(message: Message) -> Result<Self, Self::Error> {
        Err(message)
    }
}

fn update(_state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        // 1. Manejador explícito de nuestra capa invisible
        Message::CerrarMenu => {
            println!("Clic en la capa invisible. Cerrando...");
            std::process::exit(0)
        }
        
        // 2. Si hacemos clic dentro del menú, no hacemos nada
        Message::IgnorarClic => Task::none(),
        
        Message::EventOccurred(event) => match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                println!("Tecla Esc presionada. Cerrando...");
                std::process::exit(0)
            }
            Event::Window(window::Event::Unfocused) => {
                println!("Foco perdido por el sistema. Cerrando...");
                std::process::exit(0)
            }
            _ => Task::none(),
        },
    }
}

fn view(_state: &AppState) -> Element<'_, Message> {
    // 1. EL MENÚ REAL: Estrictamente de 400x300.
    // ¡Ojo! Aquí YA NO usamos center_x ni center_y.
    let menu = container(text("¡Hola, Portapapeles!").size(30))
        .width(Length::Fixed(400.0))
        .height(Length::Fixed(300.0))
        .style(|_theme| container::background(Color::from_rgb8(40, 40, 40)));

    // 2. EL ESCUDO: Si hacen clic EXACTAMENTE dentro del cuadro de 400x300, lo ignoramos
    let menu_interactivo = mouse_area(menu).on_press(Message::IgnorarClic);

    // 3. LA CAPA DE FONDO: Ocupa toda la pantalla, envuelve al menú interactivo y LO CENTRA.
    let capa_fondo = container(menu_interactivo)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        // Fondo completamente transparente
        .style(|_theme| container::background(Color::TRANSPARENT));

    // 4. EL ATRAPACLICS: Cualquier clic que caiga en la capa de fondo cierra la app
    mouse_area(capa_fondo).on_press(Message::CerrarMenu).into()
}
fn subscription(_state: &AppState) -> Subscription<Message> {
    event::listen().map(Message::EventOccurred)
}

fn main() -> Result<(), iced_layershell::Error> {
    application(|| (AppState::default(), Task::none()), "mi_popup", update, view)
        .subscription(subscription)
        .settings(Settings {
            layer_settings: LayerShellSettings {
                anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}