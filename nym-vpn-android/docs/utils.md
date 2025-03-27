# Utilities

There are a few other utilities that help support this application that are worth noting.

## Localization

[Weblate](https://weblate.nymte.ch/projects/nymvpn/) was the original localization tool but has recently \
been replaced by [Crowdin](https://crowdin.com/). Both solutions will create pull requests when new translations are available.

### Crowdin Configuration

When a new locale is added via Crowdin, it is important to ensure that the `crowdin-configs/android.yml` \
file accurately reflects the proper locale format expected by Android.

### Google Play Console / Fastlane

When new translations are added, it is important to ensure the following conditions are met to prevent Fastlane deployment failures:

- App name is ≤ 30 characters
- Short description ≤ 80 characters
- Full description ≤ 4,000 characters

These limits should be enforced by Crowdin, but they are worth noting.

## Logging

For logging, there is a local logger that writes the app logs to local storage and can optionally be shared with support.

The project was originally configured with [Sentry](https://sentry.io/auth/login/nymtech/), but we removed this from Android due to concerns about logging. Even \
when turned off, the dependencies were visible in the F-Droid store. There may be a request to add this back in the future, \
and the secrets are already available in the GitHub workflows.

Additionally, crash reports and ANRs are already available in the Google Play Console, although with less detail.
