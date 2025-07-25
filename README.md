# WebDriver Manager

A Rust library to automatically download and manage browser drivers.

# About

This library solves a common problem in web automation and tesing: ensuring the correct version of a browser driver (like `chromedriver`) is installed and available for use. It automates the process of checking the installed browser version, finding the corresponding driver version from the official sources, and downloading it to a specified location.

I designed this project since I needed a tool for some Rust applications I have been working on to automate this webdriver management since it is not the most practical process to download these for end-users. I designed the library to be reliable, fast, and easy to integrate into any Rust application, especially those using async frameworks like Tokio. It's an ideal tool for projects using libraries like `fantoccini` or `selenium` or other webautomation tools that depend on a webdriver.

# ✨ Features

- **Automatic Browser Detection ** d
