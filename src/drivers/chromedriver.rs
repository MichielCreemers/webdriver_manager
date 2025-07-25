//! [TODO] Description...

use crate::error::WebDriverError;
use crate::browser::get_browser_version;
use crate::downloader::{download_and_unzip};
use crate::WebDriverManager;
use async_trait::async_trait;
use serde::Deserialize;
use std::path::{Path, PathBuf};

// The main URL for the new JSON endpoints.
const CHROMEDRIVER_URLS_ENDPOINT: &str = 
    "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";

/// Public struct for managing Chromedriver.
pub struct ChromeDriver;

#[async_trait]
impl WebDriverManager for ChromeDriver {
    fn get_driver_name(&self) -> &str {
        "chromedriver"
    }

    async fn get_browser_version(
        &self, browser_path: Option<&Path>,
    ) -> Result<String, WebDriverError> {
        get_browser_version("chrome", browser_path).await
    }

    async fn get_driver_version(&self, browser_version: &str) -> Result<String, WebDriverError> {
        let (driver_version, _url) = get_chromedriver_download_url(browser_version).await?;
        Ok(driver_version)
    }

    async fn get_download_url(&self, driver_version: &str) -> Result<String, WebDriverError> {
        let browser_version = driver_version;
        let (_driver_version, url) = get_chromedriver_download_url(browser_version).await?;
        Ok(url)
    }

    async fn download_and_install(
        &self,
        driver_version: &str,
        install_path: &Path,
    ) -> Result<PathBuf, WebDriverError> {
        let (_driver_version, url) = get_chromedriver_download_url(driver_version).await?;

        let driver_name = self.get_driver_name();
        let driver_path = download_and_unzip(&url, install_path, driver_name).await?;

        self.verify_driver(&driver_path).await?;
        Ok(driver_path)
    }

    async fn verify_driver(&self, driver_path: &Path) -> Result<(), WebDriverError> {

        let mut command = tokio::process::Command::new(driver_path);
        command.arg("--version");

        let output = command
            .output()
            .await
            .map_err(|e| WebDriverError::CommandExecutionError { 
                command: format!("{:?}", command), 
                source: e, 
            })?;

        if !output.status.success() {
            return Err(WebDriverError::VerificationError(
                "Driver process exited with a non-zero status.".to_string(),
            ));
        }

        let stdout = String::from_utf8(output.stdout).map_err(|e| {
            WebDriverError::CommandOutputParsingError { 
                command: format!("{:?}", command), 
                source: e, 
            }
        })?;

        if !stdout.contains("ChromeDriver") {
            return Err(WebDriverError::VerificationError(format!(
                "Unexpected outpur during verificatioon: {}",
                stdout
            )));
        }

        Ok(())

    }

}

/// Represents a single download URL for a specific platform.
#[derive(Debug, Deserialize)]
struct Download {
    platform: String,
    url: String,
}

/// Represents the available downloads for a specific Chromedriver version.
#[derive(Debug, Deserialize)]
struct VersionDownloads {
    chromedriver: Option<Vec<Download>>, // must be optional, some versions have no key 'chromedriver' and serde json chrashes.
}

/// Represents a single version entry in the main JSON response.
#[derive(Debug, Deserialize)]
struct Version {
    version: String,
    downloads: VersionDownloads,
}

/// The top-level structure of the JSON response.
#[derive(Debug, Deserialize)]
struct KnownGoodVersions {
    versions: Vec<Version>,
}

/// Fetches the driver download URL for a specific *browser* version.
/// 
/// It queries the Google JSON endpoints, finds the closest matching version,
/// and returns `(driver_version, url)`
async fn get_chromedriver_download_url(
    browser_version: &str,
) -> Result<(String, String), WebDriverError> {

    // Determine the platform identifier used by Google's JSON endpoints.
    let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "win64",
        ("windows", "x86") => "win32",
        ("macos", "x86_64") => "mac-x64",
        ("macos", "aarch64") => "mac-arm64",
        ("linux", "x86_64") => "linux64",
        _ => {
            return Err(WebDriverError::UnsupportedPlatform(format!(
                "{}-{}",
                std::env::consts::OS,
                std::env::consts::ARCH
            )))
        }
    };

    // Fetch the JSON data.
    let response: KnownGoodVersions = reqwest::get(CHROMEDRIVER_URLS_ENDPOINT)
        .await?
        .json()
        .await?;

    // The browser version might be "115.0.5790.171". Then you only need "115.0.5790".
    let major_browser_version = browser_version
        .rsplitn(2, '.')
        .nth(1)
        .ok_or_else(|| WebDriverError::BrowserVersionParsingError {
            output: browser_version.to_string(),
        })?;

    // Find the latest version in the JSON that matches the major version of the browser.
    let best_match = response
        .versions
        .iter()
        .filter(|v| v.version.starts_with(major_browser_version))
        .last() // The list is sorted, so the last one is the newest patch.
        .ok_or_else(|| WebDriverError::DriverVersionNotFound {
            browser_version: browser_version.to_string(),
            platform: platform.to_string(),
        })?;

    // From that version, find the download URL for our specific platform.
    let download = best_match
        .downloads
        .chromedriver
        .as_ref() // Convert Option<Vec> to Option<&Vev> tp borrow
        .ok_or_else(|| WebDriverError::DriverUrlNotFound {
            driver_version: best_match.version.clone(),
            platform: platform.to_string(),
        })?
        .iter()
        .find(|d| d.platform == platform)
        .ok_or_else(|| WebDriverError::DriverUrlNotFound {
            driver_version: best_match.version.clone(),
            platform: platform.to_string(),
        })?;

    Ok((best_match.version.clone(), download.url.clone()))
    
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_download_url_for_known_version() {
        // Use a known good browser version to test the JSON endpoint logic.
        // This version should be new enough to likely remain in the JSON file for a long time.
        let browser_version = "138.0.7204.158";
        let result = get_chromedriver_download_url(browser_version).await;

        println!("Test Result for browser version {}: {:?}", browser_version, result);

        assert!(result.is_ok());
        let (driver_version, url) = result.unwrap();

        // Check that we got a reasonable driver version. It should be >= browser_version's major components.
        assert!(driver_version.starts_with("138.0.7204"));

        // Check that the URL is valid and contains platform info.
        assert!(url.starts_with("https://storage.googleapis.com/chrome-for-testing-public/"));
        
        let platform_str = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => "win64",
            ("windows", "x86") => "win32",
            ("macos", "x86_64") => "mac-x64",
            ("macos", "aarch64") => "mac-arm64",
            ("linux", "x86_64") => "linux64",
            _ => panic!("Unsupported test platform"),
        };
        assert!(url.contains(platform_str));
    }
}