# Research

Before building ClipCrab I wrote these throwaway experiments to learn each piece in isolation.
They are kept for reference only. They are not maintained and are not part of the ClipCrab build.
Each one is a standalone Cargo project: `cd` into it and run `cargo run`.

| Experiment | Question it explored | Outcome |
| --- | --- | --- |
| [`ClipboardListener`](ClipboardListener/) | How can I get notified when something is copied on Wayland? | `wayland-clipboard-listener` (data-control protocol) became the daemon's event source. |
| [`NotificationManager`](NotificationManager/) | How do I send desktop notifications from Rust? | Calling `notify-send` through `std::process::Command`; it became the `Pager` trait. |
| [`CircularBuffer`](CircularBuffer/) | How do I keep only the last N entries without unbounded growth? | `circular-buffer` behind a trait; it became `CircularLog`. |
| [`ClipboardInstance`](ClipboardInstance/) | Can `arboard` read and write the clipboard? | It worked, but it was unreliable on Wayland, so it was replaced by `wl-clipboard-rs` and `wl-copy`. |
| [`wl_clipboard`](wl_clipboard/) | Placeholder for the `wl-clipboard-rs` migration. | The work moved straight into the main crate. |
| [`GUI/rofi_testing`](GUI/rofi_testing/) | Can rofi in dmenu mode act as the picker? | Yes. It is the UI ClipCrab uses today. |
| [`GUI/gtk_popup`](GUI/gtk_popup/) | GTK4 and layer-shell popup. | It works, but it is a heavy dependency for a simple list. |
| [`GUI/iced_testing`](GUI/iced_testing/) | iced and layer-shell popup. | Same trade-off as GTK. |
| [`GUI/SlintTesting`](GUI/SlintTesting/) | Slint UI with a floating window. | Nice result, but it needs compositor-specific window rules. |

The conclusion was to keep the core small and rely on rofi for the UI.
A native popup could come back later behind its own trait.
