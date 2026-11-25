//! Persistence of the clipboard history in `$XDG_STATE_HOME/clipcrab/state.json`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::circular_trait::CircularLog;
use crate::storage;

const STATE_FILE: &str = "state.json";

/// Directory holding the persisted history: `$XDG_STATE_HOME/clipcrab`.
pub fn state_directory() -> io::Result<PathBuf> {
    storage::xdg_app_directory("XDG_STATE_HOME", ".local/state")
}

/// Atomically writes the history to disk with owner-only permissions.
pub fn save_state<T: CircularLog>(log: &T) -> Result<(), Box<dyn std::error::Error>> {
    save_state_to(&state_directory()?, &log.get_items())
}

/// Loads the persisted history, returning an empty list if none exists yet.
pub fn load_state() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    load_state_from(&state_directory()?)
}

fn save_state_to(directory: &Path, items: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    storage::create_private_directory(directory)?;

    let state_path = directory.join(STATE_FILE);
    let temporary_path = directory.join(format!("state.{}.tmp", std::process::id()));

    fs::write(&temporary_path, serde_json::to_vec(items)?)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&temporary_path, fs::Permissions::from_mode(0o600))?;
    }

    // Rename is atomic, so a reader never sees a half-written file.
    fs::rename(&temporary_path, &state_path)?;

    Ok(())
}

fn load_state_from(directory: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let state_path = directory.join(STATE_FILE);

    if !state_path.exists() {
        return Ok(Vec::new());
    }

    let json = fs::read_to_string(state_path)?;

    Ok(serde_json::from_str(&json)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_state_loads_as_empty() {
        let directory = tempfile::tempdir().unwrap();

        assert!(load_state_from(directory.path()).unwrap().is_empty());
    }

    #[test]
    fn state_round_trips() {
        let directory = tempfile::tempdir().unwrap();
        let state_directory = directory.path().join("clipcrab");
        let items = vec![
            "plain text".to_string(),
            "multi\nline ✓".to_string(),
            storage::history_item_for_image(Path::new("/tmp/a.png")),
        ];

        save_state_to(&state_directory, &items).unwrap();

        assert_eq!(load_state_from(&state_directory).unwrap(), items);
    }

    #[test]
    fn corrupt_state_is_an_error() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join(STATE_FILE), "not json").unwrap();

        assert!(load_state_from(directory.path()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn state_is_private_to_the_user() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let state_directory = directory.path().join("clipcrab");
        save_state_to(&state_directory, &["secret".to_string()]).unwrap();

        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(&state_directory), 0o700);
        assert_eq!(mode(&state_directory.join(STATE_FILE)), 0o600);
    }
}
