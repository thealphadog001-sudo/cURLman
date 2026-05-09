# Postman TUI

A blazing fast, terminal-based API client heavily inspired by Postman. Built with Rust, Ratatui, and libcurl.

## Features

- **TUI Interface:** Clean and responsive terminal UI with separate panes for URL, HTTP Method, Headers, Request Body, and Response.
- **Asynchronous Requests:** Powered by `libcurl` running in a background thread. Your UI will never block while waiting for a response.
- **Environment Variables:** Easily manage and swap environments using standard JSON. Supports Postman-like syntax (`{{base_url}}/api/v1/users`) for variable substitution.
- **Save to File:** Easily download and save API responses directly to your local file system.
- **State Persistence:** Automatically saves your active request and environments to `~/.config/postman_tui/state.json`.
- **Mouse Support:** Click on panes to quickly navigate.

## Prerequisites

Before building, ensure you have Rust and Cargo installed (Rust 1.88.0+ / Nightly is recommended for the 2024 edition). Additionally, since this project relies on `libcurl`, you may need to install standard development libraries for curl and openssl depending on your OS:

- **Ubuntu/Debian:** `sudo apt install libcurl4-openssl-dev libssl-dev`
- **macOS:** `brew install curl openssl` (often already available via Xcode Command Line Tools)
- **Windows:** See the [Windows Installation Guide](#windows-installation-guide) below.

## Installation

You can build and install the application directly using Cargo:

```bash
git clone <repository-url>
cd <repository-directory>
cargo install --path .
```

Alternatively, just build and run it locally:

```bash
cargo build --release
./target/release/app
```

### Windows Installation Guide

Building `curl` (and `openssl-sys`) from source on Windows can be tricky. The easiest way to get it working is by using `vcpkg`.

1. Install [vcpkg](https://github.com/microsoft/vcpkg#quick-start-windows).
2. Install `curl` and `openssl` via `vcpkg` for a 64-bit Windows target:
   ```powershell
   vcpkg install curl:x64-windows
   vcpkg install openssl:x64-windows
   ```
3. Integrate `vcpkg` with your build system and point `cargo` to the installation by setting the following environment variables (adjust `C:\vcpkg` to your actual installation path):
   ```powershell
   $env:VCPKG_ROOT="C:\vcpkg"
   $env:RUSTFLAGS="-Ctarget-feature=+crt-static"
   ```
4. Now you can build the application normally:
   ```powershell
   cargo build --release
   .\target\release\app.exe
   ```

## Usage Guidelines

Launch the app from your terminal:

```bash
app
```

### Navigation & Keybindings

- **Tab / Shift+Tab:** Cycle focus between panes (URL -> Method -> Headers -> Body -> Response).
- **Mouse Clicks:** You can also simply click on a pane to focus it.
- **Ctrl + Q:** Quit the application.
- **Esc:** Exit from editing certain panes (like the Request Body or Environments) to return to regular navigation.

### Making Requests

1. Focus the **Method** pane and type the HTTP method (e.g., `GET`, `POST`, `PUT`, `DELETE`).
2. Focus the **Headers** pane. Headers are inputted as key-value pairs separated by a double pipe (`||`).
   - Example: `Content-Type: application/json || Authorization: Bearer token123`
3. Focus the **Body** pane and write your payload (JSON, text, etc.). Press `Esc` when done editing.
4. Focus the **URL** pane, type your destination, and press **Enter** to send the request.

### Environment Variables

You can define variables that can be dynamically injected into your URLs, Headers, or Body.

1. Press **Ctrl + E** at any time to open the Environments modal.
2. The environment is defined using a JSON format. Example:
   ```json
   {
     "name": "Local Dev",
     "variables": {
       "base_url": "http://localhost:8080",
       "token": "secret_123"
     }
   }
   ```
3. Press **Ctrl + S** while in the Environments modal to save and apply the changes.
4. Now, if your URL is `{{base_url}}/api/users`, it will automatically substitute the variables before making the request.

### Downloading Responses

If you hit an API that returns a file or a large payload you want to save:

1. After receiving a response, press **Ctrl + S**.
2. A prompt will appear. Enter the desired file path (e.g., `./response.json`).
3. Press **Enter** to save the file, or **Esc** to cancel.
