//! [TODO] Description...

use crate::error::WebDriverError;
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;

pub async fn download_and_unzip(
    url: &str,
    install_path: &Path,
    driver_name: &str,
) -> Result<PathBuf, WebDriverError> {

    // --- 1. Create a temporary directory for the download.
    let temp_dir = tempfile::Builder::new()
        .prefix("webdriver-manager-")
        .tempdir()
        .map_err(|e| WebDriverError::IoError {
            path: PathBuf::from("temp"),
            source: e,
        })?;
    let temp_path = temp_dir.path();
    let archive_path = temp_path.join("driver.zip");

    // --- 2. Download the zip file to the temporary directory.
    download_file(url, &archive_path).await?;

    // --- 3. Unzip the file into the final installation directory.
    unzip_file(&archive_path, install_path).await?;

    // --- 4. Find the driver executable within the unzipped files.
    // This is necessary because archives might contain a top-level directory.
    find_driver_executable(install_path, driver_name)
}

/// Downloads a file from a given URL and saves it to a destination path.
/// 
/// This function streams the response body to a file asynchronously.
pub async fn download_file(url: &str, dest_path: &Path) -> Result<(), WebDriverError> {

    // Ensure parent directory exists.
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| WebDriverError::IoError { 
                path: parent.to_path_buf(), 
                source: e, 
            })?;
    }

    // Make the GET request.
    let response = reqwest::get(url).await?.error_for_status()?;

    // Create the destination file.
    let mut dest_file = File::create(dest_path).await.map_err(|e| WebDriverError::IoError { 
        path: dest_path.to_path_buf(), 
        source: e, 
    })?;

    // Stream the content to the file.
    let content = response.bytes().await?;
    dest_file.write_all(&content).await.map_err(|e| WebDriverError::IoError { 
        path: dest_path.to_path_buf(), 
        source: e, 
    })?;

    Ok(())
}

/// Decompresses a .zip archive to a specified directory.
/// 
/// The core zip logic is synchronous, so we wrap it in `spawn_blocking` to
/// avoid blocking the Tokio runtime.
pub async fn unzip_file(archive_path: &Path, extract_to: &Path) -> Result<(), WebDriverError> {

    let archive_path_buf = archive_path.to_path_buf();
    let extract_to_buf = extract_to.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(&archive_path_buf).map_err(|e| WebDriverError::IoError { 
            path: archive_path_buf.clone(), 
            source: e, 
        })?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| WebDriverError::ZipError { 
            path: archive_path_buf.clone(), 
            source: e, 
        })?;

        // Ensure the extraction directory exists.
        std::fs::create_dir_all(&extract_to_buf).map_err(|e| WebDriverError::IoError {
            path: extract_to_buf.clone(),
            source: e,
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| WebDriverError::ZipError {
                path: archive_path_buf.clone(),
                source: e,
            })?;

            let outpath = match file.enclosed_name() {
                Some(path) => extract_to_buf.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath).map_err(|e| WebDriverError::IoError {
                    path: outpath,
                    source: e,
                })?;

            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p).map_err(|e| WebDriverError::IoError {
                            path: p.to_path_buf(),
                            source: e,
                        })?;
                    }
                }

                let mut outfile = std::fs::File::create(&outpath).map_err(|e| WebDriverError::IoError {
                    path: outpath.clone(),
                    source: e,
                })?;

                std::io::copy(&mut file, &mut outfile).map_err(|e| WebDriverError::IoError {
                    path: outpath,
                    source: e,
                })?;
            }

            // Set permissions for executable files on Unix-like systems.
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).mapp_err(|e| WebDriverError::IoError {
                        path: outpath,
                        source: e,
                    })?;
                }
            }
        }
        Ok(())
    })
    .await
    .unwrap() // Propagate panics from the blocking task.

}

/// Searches a directory for the driver executable file.
fn find_driver_executable(search_path: &Path, driver_name: &str) -> Result<PathBuf, WebDriverError> {

    let driver_exe_name = if cfg!(target_os = "windows") {
        format!("{}.exe", driver_name)
    } else {
        driver_name.to_string()
    };

    for entry in WalkDir::new(search_path) {
        let entry = entry.map_err(|e| WebDriverError::IoError {
            path: e.path().unwrap_or(search_path).to_path_buf(),
            source: e.into_io_error().unwrap_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "WalkDir error")
            }),
        })?;
        if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
            if file_name == driver_exe_name {
                return Ok(entry.path().to_path_buf());
            }
        }
    }

    Err(WebDriverError::VerificationError(format!(
        "Could not find '{}' in the extracted files.",
        driver_exe_name
    )))
}