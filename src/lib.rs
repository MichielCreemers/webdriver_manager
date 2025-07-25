
// Top-level public modules
pub mod error;
pub mod browser;
pub mod downloader;
pub mod drivers;

pub use error::WebDriverError;

// Main public trait
use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait WebDriverManager {

    /// Gets the name of the driver (e.g., "chromedriver").
    fn get_driver_name(&self) -> &str;

    /// Gets the browser version string (e.g., "138.0.6422.113").
    /// If `browser_path` is provided, it uses that; otherwise it attempts to find the browser.
    async fn get_browser_version(&self, browser_path: Option<&PathBuf>) -> Result<String, WebDriverError>;

    /// Determines the correct driver version for a given browser version.
    async fn get_driver_version(&self, browser_version: &str) -> Result<String, WebDriverError>;

    /// Gets the download URL for the specified driver version.
    async fn get_download_url(&self, driver_version: &str) -> Result<String, WebDriverError>;
    
    /// Downloads, unzips, and verifies the web driver.
    /// Returns the path to the driver executable.
    async fn download_and_install(
        &self,
        driver_version: &str,
        install_path: &PathBuf,
    ) -> Result<PathBuf, WebDriverError>;

    /// Verifies the driver is working by attempting to start it.
    async fn verify_driver(&self, driver_path: &PathBuf) -> Result<(), WebDriverError>;
}