# WebDriver Manager

A Rust library to automatically download and manage browser drivers.

# About

This library solves a common problem in web automation and tesing: ensuring the correct version of a browser driver (like `chromedriver`) is installed and available for use. It automates the process of checking the installed browser version, finding the corresponding driver version from the official sources, and downloading it to a specified location.

I designed this project since I needed a tool for some Rust applications I have been working on to automate this webdriver management since it is not the most practical process to download these for end-users. I designed the library to be reliable, fast, and easy to integrate into any Rust application, especially those using async frameworks like Tokio. It's an ideal tool for projects using libraries like `fantoccini` or `selenium` or other webautomation tools that depend on a webdriver.

# ✨ Features

- **Automatic Browser Detection**: Finds installed Chrome/Firefox browsers on Windows, macOS, and Linux.
- **Accurate Version Matching**: Uses the official Google Chrome for Testing JSON endpoints to dinf the exact driver version that matches your installed browser.
- **Cross-Platform**: Designed and tested to work on Windows, macOS, and Linux.
- **Async First**: Built with `tokio` for non-blocking I/O, perfect for modern async Rust applications.
- **Flexible API**: Provides both a high-level `download_and_install` function fir a one-shot setup and lower-level functions for more granular control.

# ⚙ Current supported browsers/drivers

- [x] Chrome & Chromedriver
- [ ] Firefox & Geckdriver (_upcoming_)
- [ ] Edge & msedgedriver (_upcoming_)
- [ ] iedriver
- [ ] operadriver
- [ ] safaridriver (comes pre-installed on macOS but some methods might be usefull)

# 🔮 Future Plans

- **Smart Downloads 1**: Avoids re-downloading if the correct driver is already present.
- **Smart Downloads 2** Automatic retries if downloads fail.
- **GeckoDriver Support**: Implement the `WebDriverManager` trait for Firefox's `geckodriver`.
- **EdgeDriver Support**: Implement the trait for Microsoft Edge's `msedgedriver`.
- **CLI Tool**: A CLI tool of this webdriver_manager so everyone can use it cross-platform.
- **Updating**: Updating installed drivers and/or remove outdated drivers.
-

> 💡 Dont hesitate to propose some features that might be usefull!

# 🚀 Installation

Not yet pubished to `crates.io`, so the following does not work yet:

Add the following to your `Cargo.toml`:

```rust
[dependancies]
webdriver_manager = "X.X.X" ~Replace with latest version
tokio = { version = "1.46.1", features = ["full"] }

```

# 💻 Usage

## Complete Flow

This is the easiest way to ensure the correct driver is ready to use.

```rust
use webdriver_manager::drivers::chromedriver::ChromeDriver;
use webdriver_manager::{WebDriverManager, WebDriverError};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), WebDriverError> {
    // Instantiate the manager for ChromeDriver
    let manager = ChromeDriver;

    // Define the directory where you want to install the driver
    let install_dir = PathBuf::from("./drivers");
    println!("Installing driver to: {}", install_dir.display());

    // Get the installed browser version
    let browser_version = manager.get_browser_version(None).await?;
    println!("Detected Chrome version: {}", browser_version);

    // Download and install the driver that matches the browser version.
    // The `download_and_install` function treats its parameter as the browser version for the lookup.
    let driver_path = manager.download_and_install(&browser_version, &install_dir).await?;

    println!("ChromeDriver was installed to: {}", driver_path.display());

    // Now you can use this driver_path with fantoccini or other automation libraries.
    assert!(driver_path.exists());

    Ok(())
}
```

## Individual Methods

You can also use the lower-level methods for more control over the process.

```rust
use webdriver_manager::drivers::chromedriver::ChromeDriver;
use webdriver_manager::{WebDriverManager, WebDriverError};

#[tokio::main]
async fn main() -> Result<(), WebDriverError> {
    let manager = ChromeDriver;

    // 1. Get the browser version
    let browser_version = manager.get_browser_version(None).await?;
    println!("Browser version: {}", browser_version);

    // 2. Get the corresponding driver version
    let driver_version = manager.get_driver_version(&browser_version).await?;
    println!("Required driver version: {}", driver_version);

    // 3. Get the download URL for that driver version
    // Note: for chromedriver, this lookup is still based on the browser version.
    let download_url = manager.get_download_url(&browser_version).await?;
    println!("Driver download URL: {}", download_url);

    // You can now handle the download and installation manually if needed.

    Ok(())
}
```

# 📜 License

This project is licensed under the MIT License.
