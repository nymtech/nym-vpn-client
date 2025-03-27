# CI

### PR Checks

The `ci-nym-vpn-android.yml` workflow is responsible for performing CI checks on PRs.

It will check the following:
- ktLint passes
- detekt passes
- successful debug build

### Building

The `build-nym-vpn-android.yml` workflow is responsible for building the various release types of the app.

### Publishing
The `publish-nym-vpn-android.yml` workflow is responsible for publishing the app to the places we distribute the app.

- Google Play Store via [fastlane](https://fastlane.tools/) using a `general` app flavor.
- [Fdroid custom repository](https://github.com/nymtech/fdroid) via workflow dispatch using a the`fdroid` app flavor.
- [Fdroid official repository](https://gitlab.com/fdroid/fdroiddata) via our [fdroid pipeline](https://gitlab.com/fdroid/fdroiddata/-/blob/master/metadata/net.nymtech.nymvpn.yml) using the `fdroid` app flavor.
- GitHub release (which feeds the Nym website and [Obtainium](https://github.com/ImranR98/Obtainium)) using the `fdroid` app flavor.
- Accrescent store via manual upload of the `.apks` file to the [Accrescent console](https://console.accrescent.app/login) using the `fdroid` app flavor.

> **_NOTE:_** It is important that the [fdroid pipeline](https://gitlab.com/fdroid/fdroiddata/-/blob/master/metadata/net.nymtech.nymvpn.yml) stays in sync to changes that could \
builds (like go/rust/java toolchain version changes). If this occurs, a PR might need impact reproducible \
to be opened to the [Fdroid official repository](https://gitlab.com/fdroid/fdroiddata) to resolve.
