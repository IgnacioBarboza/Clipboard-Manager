use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const IMAGE_PREFIX: &str = "[Imagen] ";

pub fn image_directory() -> io::Result<PathBuf> {
    let base_directory = match std::env::var_os("XDG_CACHE_HOME") {
        Some(path) => PathBuf::from(path),
        None => {
            let home = std::env::var_os("HOME")
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;

            PathBuf::from(home).join(".cache")
        }
    };

    Ok(base_directory.join("clipcrab").join("images"))
}

pub fn initialize_image_directory() -> io::Result<PathBuf> {
    let directory = image_directory()?;

    fs::create_dir_all(&directory)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))?;
    }

    Ok(directory)
}

pub fn image_path(extension: &str) -> io::Result<PathBuf> {
    let directory = initialize_image_directory()?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();

    Ok(directory.join(format!(
        "clipcrab_img_{}_{}.{}",
        std::process::id(),
        timestamp,
        extension
    )))
}

pub fn image_path_from_history_item(item: &str) -> Option<PathBuf> {
    let path = item.strip_prefix(IMAGE_PREFIX)?;
    Some(PathBuf::from(path.trim()))
}

pub fn cleanup_orphaned_images(history: &[String]) -> io::Result<()> {
    let directory = initialize_image_directory()?;

    let referenced_images: Vec<PathBuf> = history
        .iter()
        .filter_map(|item| image_path_from_history_item(item))
        .filter_map(|path| path.canonicalize().ok())
        .collect();

    for entry in fs::read_dir(&directory)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let canonical_path = match path.canonicalize() {
            Ok(path) => path,
            Err(_) => continue,
        };

        if !referenced_images.contains(&canonical_path) {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

pub fn is_managed_image(path: &Path) -> io::Result<bool> {
    let managed_directory = image_directory()?.canonicalize()?;
    let candidate = path.canonicalize()?;

    Ok(candidate.starts_with(managed_directory))
}