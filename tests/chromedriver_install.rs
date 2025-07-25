use std::path::PathBuf;
use webdriver_manager::{drivers::chromedriver::ChromeDriver, WebDriverError, WebDriverManager};

/// This is a full integration test that simulates the end-user workflow.
#[tokio::test]
async fn test_full_chromedriver_install_flow() {
    // 1. Instantiate the manager.
    let manager = ChromeDriver;

    // 2. Define a temporary installation directory within the project's target folder.
    let install_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("tests")
        .join("chromedriver_install");

    // Clean up any previous test runs.
    if install_dir.exists() {
        std::fs::remove_dir_all(&install_dir).unwrap();
    }

    println!(
        "Test will install chromedriver to: {}",
        install_dir.display()
    );

    // 3. Get the browser version.
    let browser_version = match manager.get_browser_version(None).await {
        Ok(v) => v,
        Err(WebDriverError::BrowserNotFound) => {
            println!("Chrome not found, skipping installation test.");
            return;
        }
        Err(e) => panic!("Failed to get browser version: {:?}", e),
    };

    println!("Detected browser version: {}", browser_version);

    // 4. Download and install the driver.
    // We pass the browser version to `download_and_install` as it's used for the lookup.
    let result = manager
        .download_and_install(&browser_version, &install_dir)
        .await;

    println!("Installation result: {:?}", result);

    // 5. Assert the result.
    assert!(result.is_ok());
    let driver_path = result.unwrap();

    // Check that the driver executable actually exists at the returned path.
    assert!(driver_path.exists());
    assert!(driver_path.is_file());

    // Optional: Clean up the directory after a successful test.
    // std::fs::remove_dir_all(&install_dir).unwrap();
}