//! Description [TODO]

use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use crate::error::WebDriverError;

#[cfg(target_os = "windows")]
use std::process::Command as StdCommand;

/// Gets the version of the specified browser.
/// 
/// If `path` is provided, it will be used directly. Otherwise, the function will
/// attempt to find the browser in standard system locations.
/// On Windows, it uses PowerShell for Chrome and parses `application.ini` for Firefox.
/// On macOS and Linux, it uses the `--version` or `-V` command-line flag.
#[cfg(target_os = "windows")]
pub async fn get_browser_version(
    browser_name: &str,
    path: Option<&PathBuf>,
) -> Result<String, WebDriverError> {
    get_version_on_windows(browser_name, path).await
}

/// Gets the version of the specified browser.
/// 
/// If `path` is provided, it will be used directly. Otherwise, the function will
/// attempt to find the browser in standard system locations.
/// On macOS and Linux, it uses the `--version` or `-V` command-line flag.
#[cfg(not(target_os = "windows"))]
pub async fn get_browser_version(
    browser_name: &str,
    path: Option<&PathBuf>,
) -> Result<String, WebDriverError> {
    get_version_from_cli(browser_name, path).await
}

// --- Windows Implementation ---

#[cfg(target_os = "windows")]
async fn get_version_on_windows(
    browser_name: &str,
    path: Option<&PathBuf>,
) -> Result<String, WebDriverError> {

    if browser_name.contains("chrome") {
        let browser_path = path.cloned().or_else(|| find_on_windows("chrome")).ok_or(WebDriverError::BrowserNotFound)?;

        let command_str = format!(
            "(Get-Command '{}').Version.ToString()",
            browser_path.display()
        );

        let output = StdCommand::new("powershell")
            .args(["-c", &command_str])
            .output()
            .map_err(|e| WebDriverError::CommandExecutionError {
                command: format!("powershell -c \"{}\"", command_str),
                source: e,
            })?;

        let stdout = String::from_utf8(output.stdout).map_err(|e| WebDriverError::CommandOutputParsingError {
            command: command_str.to_string(),
            source: e,
        })?;

        let version = stdout.trim();
        if version.is_empty() {
            Err(WebDriverError::BrowserVersionParsingError { output: stdout })
        } else {
            Ok(version.to_string())
        }

    } else {

        // For Firefox, the application.ini file remains the source of truth.
        let browser_path = path.cloned().or_else(|| find_on_windows("firefox")).ok_or(WebDriverError::BrowserNotFound)?;
        let install_dir = browser_path.parent().ok_or(WebDriverError::BrowserNotFound)?;
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

// --- Unix (macOS & Linux) Implementation ---

#[cfg(not(target_os = "windows"))]
async fn get_version_from_cli(
    browser_name: &str,
    path: Option<&PathBuf>,
) -> Result<String, WebDriverError> {
    let browser_path = match path {
        Some(p) => p.clone(),
        None => find_browser_path(browser_name)?.ok_or(WebDriverError::BrowserNotFound)?,
    };

    let version_arg = if browser_name.contains("chrome") {
        "--version"
    } else {
        "-V" // Firefox uses -V or --version
    };

    let cmd_string = format!("{} {}", browser_path.display(), version_arg);
    let output = TokioCommand::new(&browser_path)
        .arg(version_arg)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| WebDriverError::CommandExecutionError {
            command: cmd_string.clone(),
            source: e,
        })?;

    let version_output =
        String::from_utf8(output.stdout).map_err(|e| WebDriverError::CommandOutputParsingError {
            command: cmd_string,
            source: e,
        })?;

    parse_version_from_string(&version_output)
}

/// Parses a version number (e.g., "138.0.6422.113") from a string
/// This is primarily for the CLI output on macOS and Linux.
fn parse_version_from_string(output: &str) -> Result<String, WebDriverError> {

    // Example outputs:
    // Chrome: "Google Chrome 138.0.6422.133"
    // Firefox: "Mozilla Firefox 126.0"

    output
        .split_whitespace()
        .find_map(|s| {
            // A simple check: does it start with a digit and contain dots?
            if s.chars().next().map_or(false, |c| c.is_ascii_digit()) && s.contains('.') {
                Some(s.to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| WebDriverError::BrowserVersionParsingError {
            output: output.to_string(),
        })
}

/// Dispatches to the correct OS-specific search function.
#[cfg(not(target_os = "windows"))]
fn find_browser_path(browser_name: &str) -> Result<Option<PathBuf>, WebDriverError> {

    if cfg!(target_os = "macos") {
        Ok(find_on_macos(browser_name))
    } else if cfg!(target_os = "linux") {
        Ok(find_on_linux(browser_name))
    } else {
        unreachable!("This function is only called on non-Windows OSes");
    }
}

// --- Platform-Specific Implementations ---

#[cfg(target_os = "windows")]
fn find_on_windows(browser_name: &str) -> Option<PathBuf> {

    let program_files = std::env::var("ProgramFiles").ok()?;
    let program_files_x86 = std::env::var("ProgramFiles(x86)").ok()?;
    let local_appdata = std::env::var("LOCALAPPDATA").ok()?;

    let (sub_path, exe_name) = if browser_name.contains("chrome") {
        ("Google\\Chrome\\Application", "chrome.exe")
    } else { // firefox
        ("Mozilla Firefox", "firefox.exe")
    };

    [program_files, program_files_x86, local_appdata]
        .into_iter()
        .map(|base| Path::new(&base).join(sub_path).join(exe_name))
        .find(|path| path.exists())
}

#[cfg(target_os = "macos")]
fn find_on_macos(browser_name: &str) -> Option<PathBuf> {

    let path_str = if browser_name.contains("chrome") {
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
    } else { // firefox
        "/Applications/Firefox.app/Contents/MacOS/firefox"
    };
    let path = PathBuf::from(path_str);
    if path.exists() { Some(path) } else { None }
}

#[cfg(target_os = "linux")]
fn find_on_linux(browser_name: &str) -> Option<PathBuf> {

    let candidates = if browser_name.contains("chrome") {
        vec!["google-chrome", "google-chrome-stable", "chromium-browser", "chromium"]
    } else { // firefox
        vec!["firefox"]
    };

    candidates.into_iter().find_map(|name| which::which(name).ok())
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
