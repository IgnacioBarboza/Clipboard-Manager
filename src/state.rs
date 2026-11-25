use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::circular_trait::CircularLog;

pub fn state_directory() -> io::Result<PathBuf> {
    let base_directory = match env::var_os("XDG_STATE_HOME") {
        Some(path) => PathBuf::from(path),
        None => {
            let home = env::var_os("HOME")
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;

            PathBuf::from(home).join(".local").join("state")
        }
    };

    Ok(base_directory.join("clipcrab"))
}

pub fn state_file() -> io::Result<PathBuf> {
    Ok(state_directory()?.join("state.json"))
}

pub fn initialize_state_directory() -> io::Result<PathBuf> {
    let directory = state_directory()?;

    fs::create_dir_all(&directory)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
    }

    Ok(directory)
}

pub fn save_state<T: CircularLog>(log: &T) -> Result<(), Box<dyn std::error::Error>> {
    let directory = initialize_state_directory()?;
    let state_path = directory.join("state.json");
    let temporary_path = directory.join(format!("state.{}.tmp", std::process::id()));

    let json = serde_json::to_vec(&log.get_items())?;

    fs::write(&temporary_path, json)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(
            &temporary_path,
            fs::Permissions::from_mode(0o600),
        )?;
    }

    fs::rename(&temporary_path, &state_path)?;

    Ok(())
}

pub fn load_state() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let state_path = state_file()?;

    if !Path::new(&state_path).exists() {
        return Ok(Vec::new());
    }

    let json = fs::read_to_string(state_path)?;
    let items = serde_json::from_str::<Vec<String>>(&json)?;

    Ok(items)
}