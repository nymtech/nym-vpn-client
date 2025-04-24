# NymVPN UI Tests

This repository contains the end-to-end UI tests for the NymVPN application using WebdriverIO and Tauri Driver.

### Prerequisites

- Node.js (v21+)
- Firefox (for macOS testing)
- Tauri CLI
- Rust and Cargo
- GeckoDriver (for macOS testing)

### Installation

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

### macOS only: Install geckodriver

```
brew install geckodriver
```

### For Linux/Windows, ensure tauri-driver is installed:

```
cargo install tauri-driver
```

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

1. **GeckoDriver issues on macOS**:

   - Don't run tests with sudo
   - Ensure Firefox is installed at `/Applications/Firefox.app`
   - Check if GeckoDriver is in PATH or at homebrew locations

2. **Firefox freezing**:

   - Make sure you're using a compatible Firefox version
   - Try running the cleanup script before starting tests

3. **Tauri-driver issues**:
   - Ensure it's installed via `cargo install tauri-driver`
   - Check if it's in your PATH

### Debugging

For more verbose logs:

```
WDIO_LOG_LEVEL=debug npm run test
```

Check the screenshots directory after failed tests to see the application state at the time of failure.
