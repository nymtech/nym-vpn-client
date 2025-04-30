# NymVPN UI Tests

This repository contains the end-to-end UI tests for the NymVPN application using WebdriverIO and Tauri Driver.

## Prerequisites

- Node.js (v21+)
- Rust and Cargo
- Protocol Buffer Compiler (`protoc`) - **Required for building the Nym VPN client**
- Tauri Driver - **Required for WebDriver tests**
- Microsoft Edge WebDriver (for Windows testing)
- Firefox (for macOS testing)
- GeckoDriver (for macOS testing)

### Installing Protocol Buffer Compiler (protoc)

#### Windows

```
choco install protoc
```

#### macOS

```
brew install protobuf
```

#### Linux

```
sudo apt-get install protobuf-compiler
```

### Installing Tauri Driver

```
cargo install tauri-driver
```

### Installing Microsoft Edge WebDriver (Windows only)

1. Download the WebDriver for your Edge version from [Microsoft Edge WebDriver](https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/)
2. Place `msedgedriver.exe` in a directory that's in your PATH (recommended: create a `Webdriver` folder in your user directory)
3. Add the directory to your PATH:
   ```
   setx PATH "%PATH%;%USERPROFILE%\Webdriver"
   ```

### macOS only: Install geckodriver

```
brew install geckodriver
```

## Installation

1. Clone the repository:

```
git clone https://github.com/nymtech/nym-vpn-client.git
cd nym-vpn-client
```

2. Install dependencies:

```
cd nym-vpn-app
npm install
```

### Install WebdriverIO dependencies

```
cd tauri-webdriver-tests
npm install -D @wdio/cli @wdio/local-runner @wdio/mocha-framework @wdio/spec-reporter
```

## Running Tests

### Run all tests

```
npm run test
```

### Run specific test files

```
# Run home page tests
npm run testlocal

# Run settings page tests
npm run testsettings

# Run support page tests
npm run testsupport

# Run location selection tests
npm run testlocation
```

### Run all tests except connection tests

```
npx wdio run wdio.conf.js --exclude ./src/tests/specs/connection.spec.js
```

### Run tests in CI mode (with mock connections)

```
npm run testci
```

## Test Structure

- `src/tests/pageobjects/`: Page Object Models
- `src/tests/specs/`: Test specifications
- `src/tests/utils/`: Helper utilities for testing

## CI/CD Integration

This project includes GitHub Actions workflows for automated testing on multiple platforms:

- Ubuntu Linux
- Windows

## Important Configuration

- macOS tests run in browser mode with Firefox and GeckoDriver
- Linux/Windows tests run against the compiled Tauri application
- Test screenshots are saved automatically on failure
- Connection tests are excluded in CI mode

## Troubleshooting

### Common Issues

1. **Protocol Buffer Compiler (protoc) issues**:

   - Make sure `protoc` is installed and in your PATH
   - On Windows, you can set the `PROTOC` environment variable to point to the executable
   - Verify installation with `protoc --version`

2. **Tauri-driver issues**:

   - Ensure it's installed via `cargo install tauri-driver`
   - Check if it's in your PATH (usually in `~/.cargo/bin/`)
   - The tests will automatically look for tauri-driver in common locations

3. **Microsoft Edge WebDriver issues (Windows)**:

   - Make sure `msedgedriver.exe` is in your PATH
   - Download the correct version from Microsoft that matches your Edge browser version
   - Place it in a directory like `C:\Users\YourUsername\Webdriver\`

4. **GeckoDriver issues on macOS**:

   - Don't run tests with sudo
   - Ensure Firefox is installed at `/Applications/Firefox.app`
   - Check if GeckoDriver is in PATH or at homebrew locations

5. **Firefox freezing**:
   - Make sure you're using a compatible Firefox version
   - Try running the cleanup script before starting tests

### Debugging

For more verbose logs:

```
WDIO_LOG_LEVEL=debug npm run test
```

Check the screenshots directory after failed tests to see the application state at the time of failure.
