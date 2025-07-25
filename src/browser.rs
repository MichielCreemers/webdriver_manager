//! Description [TODO]

use std::path::{Path, PathBuf};
use std::process::Command;
use crate::error::WebDriverError;

#[cfg(target_os = "windows")]
use std::process::Command as StdCommand;

/// Gets the version of the specified browser.
/// 
/// If `path` is provided, it will be used directly. Otherwise, the function will
/// attempt to find the browser in standard system locations.
/// On Windows, it uses PowerShell for Chrome and parses `application.ini` for Firefox.
/// On macOS and Linux, it uses the `--version` or `-V` command-line flag.
pub async fn get_browser_version(
    browser_name: &str,
    path_override: Option<&Path>,
) -> Result<String, WebDriverError> {
    let path = match path_override {
        Some(p) => p.to_path_buf(),
        None => find_browser_path(browser_name).ok_or(WebDriverError::BrowserNotFound)?,
    };
    get_version_on_platform(browser_name, &path).await
}

/// Gets the version of the specified browser.
fn find_browser_path(browser_name: &str) -> Option<PathBuf> {
    if browser_name != "chrome" && browser_name != "firefox" {
        return None;
    }

    find_browser_path_system(browser_name)
}

// --- Platform-Specific Implementations ---

#[cfg(target_os = "windows")]
fn find_browser_path_system(browser_name: &str) -> Option<PathBuf> {
    let program_files = std::env::var("ProgramFiles").ok()?;
    let program_files_x86 = std::env::var("ProgramFiles(x86)").ok()?;
    let local_appdata = std::env::var("LOCALAPPDATA").ok()?;

    let (sub_path, exe_name) = if browser_name.contains("chrome") {
        ("Google\\Chrome\\Application", "chrome.exe")
    } else {
        // firefox
        ("Mozilla Firefox", "firefox.exe")
    };

    [program_files, program_files_x86, local_appdata]
        .into_iter()
        .map(|base| Path::new(&base).join(sub_path).join(exe_name))
        .find(|path| path.exists())
}

#[cfg(target_os = "macos")]
fn find_browser_path_system(browser_name: &str) -> Option<PathBuf> {
    let path_str = if browser_name.contains("chrome") {
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
    } else {
        // firefox
        "/Applications/Firefox.app/Contents/MacOS/firefox"
    };
    let path = PathBuf::from(path_str);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
fn find_browser_path_system(browser_name: &str) -> Option<PathBuf> {
    let candidates = if browser_name.contains("chrome") {
        vec![
            "google-chrome",
            "google-chrome-stable",
            "chromium-browser",
            "chromium",
        ]
    } else {
        // firefox
        vec!["firefox"]
    };

    candidates
        .into_iter()
        .find_map(|name| which::which(name).ok())
}

#[cfg(target_os = "windows")]
async fn get_version_on_platform(
    browser_name: &str,
    path: &Path,
) -> Result<String, WebDriverError> {
    if browser_name.contains("chrome") {
        let command_str = format!(
            "(Get-Command '{}').Version.ToString()",
            path.to_string_lossy()
        );
        let output = Command::new("powershell")
            .args(["-Command", &command_str])
            .output()
            .map_err(|e| WebDriverError::CommandExecutionError {
                command: command_str.clone(),
                source: e,
            })?;

        let version = String::from_utf8(output.stdout).map_err(|e| {
            WebDriverError::CommandOutputParsingError {
                command: command_str,
                source: e,
            }
        })?;
        Ok(version.trim().to_string())
    } else {
        // For Firefox, reading application.ini is most reliable on Windows
        let install_dir = path.parent().ok_or(WebDriverError::BrowserNotFound)?;
        let ini_path = install_dir.join("application.ini");

        if !ini_path.exists() {
            return Err(WebDriverError::IoError {
                path: ini_path,
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
            });
        }

        let content = std::fs::read_to_string(&ini_path).map_err(|e| WebDriverError::IoError {
            path: ini_path,
            source: e,
        })?;

        content
            .lines()
            .find(|line| line.starts_with("Version="))
            .and_then(|line| line.split('=').nth(1))
            .map(|s| s.to_string())
            .ok_or_else(|| WebDriverError::BrowserVersionParsingError { output: content })
    }
}

#[cfg(not(target_os = "windows"))]
async fn get_version_on_platform(
    browser_name: &str,
    path: &Path,
) -> Result<String, WebDriverError> {
    get_version_from_cli(browser_name, path)
}

#[cfg(not(target_os = "windows"))]
async fn get_version_on_platform(
    browser_name: &str,
    path: &Path,
) -> Result<String, WebDriverError> {
    get_version_from_cli(browser_name, path)
}


async fn get_version_from_cli(
    browser_name: &str,
    path: &Path,
) -> Result<String, WebDriverError> {
    let version_arg = if browser_name.contains("chrome") {
        "--version"
    } else {
        // Firefox uses -V or --version on non-windows
        "-V"
    };

    let output = Command::new(path)
        .arg(version_arg)
        .output()
        .map_err(|e| WebDriverError::CommandExecutionError {
            command: format!("'{}' {}", path.to_string_lossy(), version_arg),
            source: e,
        })?;

    let version_str = String::from_utf8(output.stdout).map_err(|e| {
        WebDriverError::CommandOutputParsingError {
            command: format!("'{}' {}", path.to_string_lossy(), version_arg),
            source: e,
        }
    })?;

    version_str
        .split_whitespace()
        .find_map(|s| {
            if s.chars().next().map_or(false, |c| c.is_ascii_digit()) && s.contains('.') {
                Some(s.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| WebDriverError::BrowserVersionParsingError {
            output: version_str,
        })
}

// --- Tests ---
// Run these tests with `cargo test -- --nocapture`
#[cfg(test)]
mod tests {
    use super::*;

    // This test will run and attempt to find your installed Chrome version.
    // It will be skipped if the function returns a BrowserNotFound error.
    #[tokio::test]
    async fn test_get_chrome_version() {
        match get_browser_version("chrome", None).await {
            Ok(version_string) => {
                println!("Successfully detected Chrome version: {}", version_string);
                assert!(!version_string.is_empty());
                assert!(version_string.contains('.'));
            }
            Err(WebDriverError::BrowserNotFound) => {
                println!("Chrome not found, skipping test.");
            }
            Err(e) => {
                panic!("An unexpected error occurred: {:?}", e);
            }
        }
    }

    // This test will run and attempt to find your installed Firefox version.
    // It will be skipped if the function returns a BrowserNotFound error.
    #[tokio::test]
    async fn test_get_firefox_version() {
        match get_browser_version("firefox", None).await {
            Ok(version_string) => {
                println!("Successfully detected Firefox version: {}", version_string);
                assert!(!version_string.is_empty());
                assert!(version_string.contains('.'));
            }
            Err(WebDriverError::BrowserNotFound) => {
                println!("Firefox not found, skipping test.");
            }
            Err(e) => {
                panic!("An unexpected error occurred: {:?}", e);
            }
        }
    }
}
