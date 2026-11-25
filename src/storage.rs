//! On-disk storage for copied images and XDG base-directory resolution.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Prefix used in the history to mark an entry as an image stored on disk.
///
/// The value is persisted in `state.json`, so changing it would orphan
/// existing history entries.
pub const IMAGE_PREFIX: &str = "[Imagen] ";

/// Resolves an XDG base directory (`$XDG_*_HOME`, or `$HOME/<fallback>`) and
/// appends the `clipcrab` application folder.
pub fn xdg_app_directory(variable: &str, home_fallback: &str) -> io::Result<PathBuf> {
    let base_directory = match env::var_os(variable).map(PathBuf::from) {
        // The XDG spec says relative paths must be ignored.
        Some(path) if path.is_absolute() => path,
        _ => {
            let home = env::var_os("HOME")
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;

            PathBuf::from(home).join(home_fallback)
        }
    };

    Ok(base_directory.join("clipcrab"))
}

/// Creates `directory` (and its parents) and restricts it to the current user.
pub fn create_private_directory(directory: &Path) -> io::Result<()> {
    fs::create_dir_all(directory)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))?;
    }

    Ok(())
}

/// Directory where copied images are stored: `$XDG_CACHE_HOME/clipcrab/images`.
pub fn image_directory() -> io::Result<PathBuf> {
    Ok(xdg_app_directory("XDG_CACHE_HOME", ".cache")?.join("images"))
}

/// Returns a fresh, unique path for a new image with the given file extension.
pub fn image_path(extension: &str) -> io::Result<PathBuf> {
    let directory = image_directory()?;
    create_private_directory(&directory)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos();

    Ok(directory.join(format!(
        "clipcrab_img_{}_{}.{}",
        std::process::id(),
        timestamp,
        extension
    )))
}

/// Builds the history entry that references an image saved at `path`.
pub fn history_item_for_image(path: &Path) -> String {
    format!("{IMAGE_PREFIX}{}", path.display())
}

/// Extracts the image path from a history entry, or `None` for text entries.
pub fn image_path_from_history_item(item: &str) -> Option<PathBuf> {
    let path = item.strip_prefix(IMAGE_PREFIX)?;
    Some(PathBuf::from(path.trim()))
}

/// Deletes images in the image directory that are no longer referenced by the history.
pub fn cleanup_orphaned_images(history: &[String]) -> io::Result<()> {
    let directory = image_directory()?;
    create_private_directory(&directory)?;

    cleanup_orphaned_images_in(&directory, history)
}

fn cleanup_orphaned_images_in(directory: &Path, history: &[String]) -> io::Result<()> {
    let referenced_images: Vec<PathBuf> = history
        .iter()
        .filter_map(|item| image_path_from_history_item(item))
        .filter_map(|path| path.canonicalize().ok())
        .collect();

    for entry in fs::read_dir(directory)? {
        let path = entry?.path();

        if !path.is_file() {
            continue;
        }

        let Ok(canonical_path) = path.canonicalize() else {
            continue;
        };

        if !referenced_images.contains(&canonical_path) {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

/// Returns `true` if `path` lives inside ClipCrab's image directory.
///
/// Used to refuse copying arbitrary files referenced by a tampered history.
pub fn is_managed_image(path: &Path) -> io::Result<bool> {
    is_inside(&image_directory()?, path)
}

fn is_inside(directory: &Path, path: &Path) -> io::Result<bool> {
    let managed_directory = directory.canonicalize()?;
    let candidate = path.canonicalize()?;

    Ok(candidate.starts_with(managed_directory))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_history_item_round_trips() {
        let path = Path::new("/home/user/.cache/clipcrab/images/clipcrab_img_1_2.png");
        let item = history_item_for_image(path);

        assert!(item.starts_with(IMAGE_PREFIX));
        assert_eq!(image_path_from_history_item(&item).as_deref(), Some(path));
    }

    #[test]
    fn text_items_are_not_images() {
        assert_eq!(image_path_from_history_item("hello world"), None);
        assert_eq!(image_path_from_history_item("Imagen /tmp/a.png"), None);
    }

    #[test]
    fn cleanup_keeps_referenced_and_removes_orphans() {
        let directory = tempfile::tempdir().unwrap();
        let kept = directory.path().join("kept.png");
        let orphan = directory.path().join("orphan.png");
        fs::write(&kept, b"kept").unwrap();
        fs::write(&orphan, b"orphan").unwrap();

        let history = vec!["some text".to_string(), history_item_for_image(&kept)];
        cleanup_orphaned_images_in(directory.path(), &history).unwrap();

        assert!(kept.exists());
        assert!(!orphan.exists());
    }

    #[test]
    fn detects_paths_outside_managed_directory() {
        let managed = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let inside_file = managed.path().join("image.png");
        let outside_file = outside.path().join("secret.png");
        fs::write(&inside_file, b"").unwrap();
        fs::write(&outside_file, b"").unwrap();

        assert!(is_inside(managed.path(), &inside_file).unwrap());
        assert!(!is_inside(managed.path(), &outside_file).unwrap());
        // `..` segments must not escape the managed directory.
        let escaped = managed
            .path()
            .join("..")
            .join(outside.path().file_name().unwrap())
            .join("secret.png");
        assert!(!is_inside(managed.path(), &escaped).unwrap());
    }
}
