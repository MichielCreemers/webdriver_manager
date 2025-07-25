//! [TODO] Description...

use crate::error::WebDriverError;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

// The main URL for the new JSON endpoints.
const CHROMEDRIVER_URLS_ENDPOINT: &str = 
    "https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json";

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

/// Fetches the driver download URL for a specific browser version.
/// 
/// It queries the Google JSON endpoints, finds the closest matching version,
/// and returns the download URL for the correct platform.
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