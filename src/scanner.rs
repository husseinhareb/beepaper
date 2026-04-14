//! Wallpaper directory scanning and image filtering.

use std::path::{Path, PathBuf};

use anyhow::Result;
use walkdir::WalkDir;

/// Normalize configured file extensions to lowercase values without leading dots.
pub fn normalize_extensions(extensions: &[String]) -> Vec<String> {
    let mut normalized: Vec<String> = extensions
        .iter()
        .map(|ext| ext.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
        .collect();

    normalized.sort();
    normalized.dedup();
    normalized
}

/// Return `true` when a file path has one of the configured image extensions.
pub fn is_supported_image(path: &Path, extensions: &[String]) -> bool {
    let Some(extension) = path.extension().and_then(|value| value.to_str()) else {
        return false;
    };

    let normalized = extension.to_ascii_lowercase();
    extensions.iter().any(|candidate| candidate == &normalized)
}

/// Scan configured directories for matching image files.
pub fn scan_directories(
    dirs: &[PathBuf],
    recursive: bool,
    extensions: &[String],
) -> Result<Vec<PathBuf>> {
    let normalized_extensions = normalize_extensions(extensions);
    let mut files = Vec::new();

    for dir in dirs {
        if !dir.exists() {
            continue;
        }

        let walker = if recursive {
            WalkDir::new(dir)
        } else {
            WalkDir::new(dir).max_depth(1)
        };

        for entry in walker.into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if is_supported_image(path, &normalized_extensions) {
                files.push(path.to_path_buf());
            }
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::{is_supported_image, normalize_extensions, scan_directories};
    use anyhow::Result;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn supported_extension_filter_is_case_insensitive() {
        let extensions = normalize_extensions(&["png".to_string(), ".JPG".to_string()]);

        assert!(is_supported_image(Path::new("image.PNG"), &extensions));
        assert!(is_supported_image(Path::new("image.jpg"), &extensions));
        assert!(!is_supported_image(Path::new("notes.txt"), &extensions));
        assert!(!is_supported_image(Path::new("no_extension"), &extensions));
    }

    #[test]
    fn non_recursive_scan_ignores_nested_files() -> Result<()> {
        let root = tempdir()?;
        let nested = root.path().join("nested");

        fs::create_dir_all(&nested)?;
        fs::write(root.path().join("top.jpg"), b"test")?;
        fs::write(root.path().join("ignore.txt"), b"test")?;
        fs::write(nested.join("deep.png"), b"test")?;

        let files = scan_directories(
            &[root.path().to_path_buf()],
            false,
            &["jpg".into(), "png".into()],
        )?;

        assert_eq!(files, vec![root.path().join("top.jpg")]);
        Ok(())
    }

    #[test]
    fn recursive_scan_includes_nested_files() -> Result<()> {
        let root = tempdir()?;
        let nested = root.path().join("nested");

        fs::create_dir_all(&nested)?;
        fs::write(root.path().join("top.jpg"), b"test")?;
        fs::write(nested.join("deep.png"), b"test")?;

        let files = scan_directories(
            &[root.path().to_path_buf()],
            true,
            &["jpg".into(), "png".into()],
        )?;

        let mut expected = vec![root.path().join("top.jpg"), nested.join("deep.png")];
        expected.sort();

        assert_eq!(files, expected);
        Ok(())
    }
}
