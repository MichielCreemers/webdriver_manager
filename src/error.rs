use thiserror::Error;
use std::path::PathBuf;
use std::io;

/// Error type for all possible failures in the library.
#[derive(Error, Debug)]
pub enum WebDriverError {
    #[error("Failed to execute command '{command}': {source}")]
    CommandExecutionError {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Command '{command}' output could not be parsed: {source}")]
    CommandOutputParsingError {
        command: String,
        #[source]
        source: std::string::FromUtf8Error,
    },

    #[error("Browser not found. Please specify the path manually or ensure it's in a standard location.")]
    BrowserNotFound,

    #[error("Failed to parse browser version from output: '{output}'")]
    BrowserVersionParsingError {
        output: String,
    },

    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Failed to parse JSON response from '{url}': {source}")]
    JsonParseError {
        url: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Could not find a matching driver version for browser version '{browser_version}' on platform '{platform}'")]
    DriverVersionNotFound {
        browser_version: String,
        platform: String,
    },

    #[error("Could not find a download URL for driver version {driver_version} on platform {platform}")]
    DriverUrlNotFound {
        driver_version: String,
        platform: String,
    },
    
    #[error("I/O error accessing path '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to decompress zip file to '{path}': {source}")]
    ZipError {
        path: PathBuf,
        #[source]
        source: zip::result::ZipError,
    },

    #[error("Driver executable not found in the downloaded archive at '{path}'")]
    DriverExecutableNotFound {
        path: PathBuf,
    },
    
    #[error("Failed to start driver at '{path}': {source}")]
    DriverVerificationError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("An unknown error has occurred: {0}")]
    Custom(String),

    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),

    #[error("An error occurred while verifying the driver")]
    VerificationError(String),
}