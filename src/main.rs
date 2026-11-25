//! ClipCrab: a lightweight clipboard history manager for Wayland.
//!
//! Run without arguments to start the daemon that records clipboard history,
//! and with `--menu` to pick an entry from the history through rofi.

mod circular_impl;
mod circular_trait;
mod instance_impl;
mod instance_trait;
mod pager_impl;
mod pager_trait;
mod state;
mod storage;

use std::env;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

use wayland_clipboard_listener::{WlClipboardPasteStream, WlListenType};

use circular_impl::Buffer;
use circular_trait::CircularLog;
use instance_impl::WaylandClipboard;
use instance_trait::ClipboardInstance;
use pager_impl::SystemPager;
use pager_trait::Pager;

type AppResult<T = ()> = Result<T, Box<dyn Error>>;

const USAGE: &str = "\
ClipCrab - a lightweight clipboard history manager for Wayland

Usage:
  clipcrab            Start the daemon that records clipboard history
  clipcrab --menu     Open the history in rofi and copy the selected entry

Options:
  -h, --help          Print this help
  -V, --version       Print version information
";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => run_daemon(),
        ["--menu"] => run_menu(),
        ["-h" | "--help"] => {
            print!("{USAGE}");
            Ok(())
        }
        ["-V" | "--version"] => {
            println!("clipcrab {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => {
            eprint!(
                "error: unrecognized arguments: {}\n\n{USAGE}",
                args.join(" ")
            );
            return ExitCode::from(2);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("clipcrab: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Loads the persisted history into a fresh buffer. A missing or unreadable
/// state file is reported and treated as an empty history.
fn load_history() -> Buffer {
    let mut log = Buffer::new();

    match state::load_state() {
        Ok(items) => items.into_iter().for_each(|item| log.push(item)),
        Err(error) => eprintln!("Failed to load clipboard state: {error}"),
    }

    log
}

/// Adds `item` as the newest history entry (moving it up if it was already
/// present) and persists the history.
fn record_history_item(log: &mut Buffer, item: String) {
    push_unique(log, item);

    if let Err(error) = state::save_state(log) {
        eprintln!("Failed to save clipboard state: {error}");
    }
}

/// Pushes `item` as the newest entry, removing an earlier copy of it first so
/// the history never holds duplicates.
fn push_unique<T: CircularLog>(log: &mut T, item: String) {
    if let Some(index) = log.get_items().iter().position(|stored| stored == &item) {
        let _ = log.remove_index(index);
    }

    log.push(item);
}

// ---------------------------------------------------------------------------
// Menu mode
// ---------------------------------------------------------------------------

fn run_menu() -> AppResult {
    let log = load_history();
    let items = log.get_items();

    if items.is_empty() {
        println!("The clipboard history is empty.");
        return Ok(());
    }

    let Some(selected_index) = show_rofi_menu(&items)? else {
        return Ok(());
    };

    let selected = items[selected_index].clone();
    let mut clipboard = WaylandClipboard::new()?;

    if let Some(image_path) = storage::image_path_from_history_item(&selected) {
        if !storage::is_managed_image(&image_path).unwrap_or(false) {
            return Err(format!(
                "Refusing to copy {}: it is not a ClipCrab image (missing or outside the cache directory)",
                image_path.display()
            )
            .into());
        }

        clipboard.write_image(&image_path.to_string_lossy())?;
    } else {
        clipboard.write_text(selected)?;
    }

    Ok(())
}

/// Builds the rofi dmenu input: newest entry first, numbered from 1.
///
/// Newlines are shown as `↵` and image entries use rofi's icon protocol
/// (`label\0icon\x1fpath`) so a thumbnail is displayed.
fn build_rofi_input(items: &[String]) -> String {
    let mut input = String::new();

    for (position, item) in items.iter().rev().enumerate() {
        let number = position + 1;

        if let Some(image_path) = storage::image_path_from_history_item(item) {
            let file_name = image_path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Image".to_string());

            input.push_str(&format!(
                "{number}: {file_name}\0icon\x1f{}\n",
                image_path.display()
            ));
        } else {
            input.push_str(&format!("{number}: {}\n", item.replace('\n', " ↵ ")));
        }
    }

    input
}

/// Maps the line rofi printed back to an index into the history (oldest first).
fn parse_rofi_selection(selected: &str, item_count: usize) -> Option<usize> {
    let (number, _) = selected.trim().split_once(':')?;
    let display_index = number.parse::<usize>().ok()?;

    if display_index == 0 || display_index > item_count {
        return None;
    }

    Some(item_count - display_index)
}

/// Shows the history in rofi and returns the index of the chosen entry, or
/// `None` if the menu was dismissed.
fn show_rofi_menu(items: &[String]) -> AppResult<Option<usize>> {
    let mut child = Command::new("rofi")
        .args(["-dmenu", "-i", "-p", "ClipCrab", "-show-icons"])
        .arg("-theme-str")
        .arg(
            "window { width: 35%; } listview { lines: 6; } element-icon { size: 3ch; } \
             element { children: [ element-text, element-icon ]; }",
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute rofi (is it installed and in PATH?): {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(build_rofi_input(items).as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        // rofi exits with a non-zero status when the menu is dismissed.
        return Ok(None);
    }

    Ok(parse_rofi_selection(
        &String::from_utf8_lossy(&output.stdout),
        items.len(),
    ))
}

// ---------------------------------------------------------------------------
// Daemon mode
// ---------------------------------------------------------------------------

fn run_daemon() -> AppResult {
    let mut stream = WlClipboardPasteStream::init(WlListenType::ListenOnCopy).map_err(|e| {
        format!(
            "Failed to connect to the Wayland clipboard ({e:?}). \
             ClipCrab needs a Wayland compositor with the data-control protocol."
        )
    })?;

    let mut clipboard_log = load_history();

    if let Err(error) = storage::cleanup_orphaned_images(&clipboard_log.get_items()) {
        eprintln!("Failed to clean orphaned images: {error}");
    }

    for event in stream.paste_stream().flatten() {
        let content = event.context;

        // Images are stored on disk and the history keeps a reference to the file.
        if let Some(image_mime) = event.mime_types.iter().find(|m| m.starts_with("image/")) {
            match save_clipboard_image(image_mime) {
                Ok(path) => {
                    record_history_item(&mut clipboard_log, storage::history_item_for_image(&path))
                }
                Err(error) => eprintln!("Failed to save copied image: {error}"),
            }
            continue;
        }

        match content.mime_type.as_str() {
            "text/plain;charset=utf-8" | "text/plain" | "application/octet-stream" => {
                if let Ok(text) = String::from_utf8(content.context) {
                    record_history_item(&mut clipboard_log, text);
                }
            }
            unsupported => {
                let _ = SystemPager::notify_user(
                    "New Input in the Clipboard",
                    &format!("Not Supported for {unsupported} type"),
                );
            }
        }
    }

    Ok(())
}

/// Reads the current clipboard image of type `mime_type` and saves it in the
/// image cache, returning the path of the new file.
fn save_clipboard_image(mime_type: &str) -> AppResult<std::path::PathBuf> {
    let image_bytes = WaylandClipboard::new()?.read_image(mime_type)?;

    let extension = mime_type.split('/').next_back().unwrap_or("png");
    let path = storage::image_path(extension)?;
    write_private_file(&path, &image_bytes)?;

    Ok(path)
}

fn write_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    fs::write(path, bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn rofi_input_lists_newest_first() {
        let input = build_rofi_input(&items(&["oldest", "middle", "newest"]));

        assert_eq!(input, "1: newest\n2: middle\n3: oldest\n");
    }

    #[test]
    fn rofi_input_flattens_multiline_text() {
        let input = build_rofi_input(&items(&["line one\nline two"]));

        assert_eq!(input, "1: line one ↵ line two\n");
    }

    #[test]
    fn rofi_input_uses_icon_protocol_for_images() {
        let image = storage::history_item_for_image(Path::new("/cache/clipcrab/images/shot.png"));
        let input = build_rofi_input(&[image]);

        assert_eq!(
            input,
            "1: shot.png\0icon\x1f/cache/clipcrab/images/shot.png\n"
        );
    }

    #[test]
    fn rofi_selection_maps_to_history_index() {
        // Three items: display "1" is the newest, which is the last stored index.
        assert_eq!(parse_rofi_selection("1: newest\n", 3), Some(2));
        assert_eq!(parse_rofi_selection("3: oldest", 3), Some(0));
        assert_eq!(parse_rofi_selection("2: text: with colons", 3), Some(1));
    }

    #[test]
    fn rofi_selection_rejects_invalid_lines() {
        assert_eq!(parse_rofi_selection("0: nothing", 3), None);
        assert_eq!(parse_rofi_selection("4: out of range", 3), None);
        assert_eq!(parse_rofi_selection("typed by the user", 3), None);
        assert_eq!(parse_rofi_selection("abc: def", 3), None);
        assert_eq!(parse_rofi_selection("", 3), None);
    }

    #[test]
    fn push_unique_moves_duplicates_to_the_top() {
        let mut log = Buffer::new();
        for value in ["a", "b", "c", "a"] {
            push_unique(&mut log, value.to_string());
        }

        assert_eq!(log.get_items(), items(&["b", "c", "a"]));
    }

    #[test]
    fn push_unique_appends_new_items() {
        let mut log = Buffer::new();
        push_unique(&mut log, "a".to_string());
        push_unique(&mut log, "b".to_string());

        assert_eq!(log.get_items(), items(&["a", "b"]));
    }
}
