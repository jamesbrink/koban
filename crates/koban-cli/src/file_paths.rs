use std::{fs, path::Path};

use koban::{KobanError, Result};

pub(crate) fn write_download_file(path: &Path, bytes: Vec<u8>, force: bool) -> Result<()> {
    ensure_download_path(path, force)?;
    fs::write(path, bytes).map_err(|source| KobanError::File {
        message: source.to_string(),
    })
}

pub(crate) fn ensure_download_path(path: &Path, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Err(KobanError::File {
            message: format!("{} already exists", path.display()),
        });
    }

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        return Err(KobanError::File {
            message: format!("parent directory {} does not exist", parent.display()),
        });
    }

    Ok(())
}

pub(crate) fn ensure_upload_file(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path).map_err(|source| KobanError::File {
        message: format!("could not read {}: {source}", path.display()),
    })?;
    if !metadata.is_file() {
        return Err(KobanError::File {
            message: format!("{} is not a file", path.display()),
        });
    }
    Ok(())
}
