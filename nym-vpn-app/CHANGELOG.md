# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.13.0] - 2025-07-30

### Added

- Add an option in the settings allowing to disable IPv6 support
- Add a new setting menu "Privacy and data" to control error monitoring
  and network statistics collection
- Remove libfuse2 dependency (EOL) on Linux AppImage

## [1.12.0] - 2025-07-18

### Added

- Add Sentry error monitoring at daemon level, controlled by
  the existing toggle in app settings (OFF by default)

### Changed

- Add additional system information in logs to help with debugging
- Update dependencies to latest versions

## [1.11.0] - 2025-06-18

### Added

- [Windows] Add auto-updater
