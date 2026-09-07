use circular_buffer::FixedCircularBuffer;
use slint::{ModelRc, VecModel, SharedString};
use std::rc::Rc;
use std::process::Command;

slint::include_modules!();

/*

hyprland.lua config:

-- Clipboard Manager (Slint)
hl.window_rule({
  name = "Slint_Clipboard_Manager",
  match = {
    title = "^(ClipboardManagerApp)$"
  },
  float = true,
  size = {"100%", "100%"},
  move = {"0", "0"}
})

*/

fn get_mouse_position() -> (f32, f32) {
    if let Ok(output) = Command::new("hyprctl").arg("cursorpos").output() {
        let pos_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = pos_str.trim().split(',').collect();
        if parts.len() == 2 {
            let x = parts[0].trim().parse::<f32>().unwrap_or(0.0);
            let y = parts[1].trim().parse::<f32>().unwrap_or(0.0);
            return (x, y);
        }
    }
    (0.0, 0.0) // Fallback
}

fn main() -> Result<(), slint::PlatformError> {    
    //Create buffer for testing
    let mut buffer = FixedCircularBuffer::<String, 5>::new();
    buffer.push_back("hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh".to_string());
    buffer.push_back("testing 1".to_string());
    buffer.push_back("testing 2".to_string());
    buffer.push_back("testing 3".to_string());
    buffer.push_back("testing 4".to_string());

    
    // Extract data from the buffer and convert them to a slint format (SharedString)
    let mut slint_items: Vec<SharedString> = Vec::new();
    
    for item in buffer.iter() {
        slint_items.push(SharedString::from(item));
    }

    //Create Window
    let ui = AppWindow::new().unwrap();

    // Instanciate model and inject to the property buffer_items
    let model = Rc::new(VecModel::from(slint_items));
    ui.set_buffer_items(ModelRc::from(model.clone()));

    //When the user clicks, the function is called
    ui.on_item_clicked(move |id| {
        println!("The id {} was selected.", id);
        slint::quit_event_loop().unwrap();
    });

    ui.on_close_requested({
        move || {
            slint::quit_event_loop().unwrap();
        }
    });
 
    ui.run()
}