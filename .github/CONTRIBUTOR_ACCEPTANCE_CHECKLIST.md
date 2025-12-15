# External Contributor Acceptance Checklist

This document outlines the requirements that all external contributors must complete before submitting a pull request to the Nym VPN Client repository. Following this checklist keeps contributions aligned with project quality standards.

## Code Quality and Compilation

All code must build without warnings across supported platforms. Run `cargo build --workspace --all-targets`, `cargo fmt --all` (use nightly if required), and `cargo clippy --workspace --all-targets -- -D warnings`. Use Rust lastest or as specified in the workspace Cargo.toml. For TypeScript/JavaScript, ensure project lint and type checks pass.

## Testing Requirements

Add tests for new behavior and regression coverage for fixes. All existing tests must pass: run `cargo test --workspace` (or the equivalent commands for other components). Include integration tests when changes span components or platform-specific paths (macOS/iOS/Android/Windows/Linux, firewall, network config, VPN tunnel handling). If mobile tests require manual steps, document how to run them.

## Documentation and Evidence

Document user-facing changes and public APIs; update README/setup notes when behavior changes. For UI or UX changes, include screenshots or short recordings showing success and error paths on affected platforms. For backend or infrastructure changes, provide a brief description or diagram of the new flow and any platform-specific considerations.

## Component Impact Analysis

State which components and files you touched, what changed, and why. Call out new or modified dependencies and their justification. If changes span core, platform-specific implementations, or the app layer, explain how they fit together and which platforms are affected.

## Functional Benefits and Justification

Explain the benefit: which problem it solves, who gains, and how it improves UX, security, privacy, or performance. Reference the related issue or ticket; include benchmarks or measurements for performance changes.

## Workflow Compliance

Your changes must pass all CI workflows: build, fmt, clippy, tests, and any platform-specific pipelines (macOS, iOS, Android, Windows, Linux, Tauri). Run the same commands locally to match CI.

## Code Review Readiness

In your PR description, confirm this checklist, link tests/results and screenshots when relevant, and be ready to explain design choices. Keep code self-explanatory with clear naming; highlight any platform-specific considerations.

## Acceptance Criteria Summary

A PR is ready when it builds and lints cleanly, passes tests, documents behavior and impact, justifies benefits, meets all CI workflows, and includes the evidence and links needed for fast review.

