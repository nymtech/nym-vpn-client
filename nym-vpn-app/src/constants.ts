import { Country } from './types';

export const AppName = 'NymVPN';
// ⚠ keep this default in sync with the one declared in
// src-tauri/src/states/app.rs
export const DefaultVpnMode = 'TwoHop';
export const ConnectionEvent = 'connection-state';
export const ErrorEvent = 'error';
export const ProgressEvent = 'connection-progress';
export const DaemonEvent = 'vpnd-status';
export const StatusUpdateEvent = 'status-update';
// ⚠ keep this value in sync with the one declared in `index.html`
export const DefaultRootFontSize = 14; // in px
// NOTE: when fresh country data is get from daemon, the selected countries
// are checked against it and if needed it is automatically switched to
// available ones
export const DefaultCountry: Country = {
  name: 'Switzerland',
  code: 'CH',
};
// TODO disabled Fastest location until the backend is ready
export const FastestFeatureEnabled = false;
export const DefaultThemeMode = 'System';

// Various external links
export const GitHubIssuesUrl =
  'https://nym.com/go/github/nym-vpn-client/issues';
export const MatrixRoomUrl = 'https://nym.com/go/matrix';
export const DiscordInviteUrl = 'https://nym.com/go/discord';
export const FaqUrl = 'https://support.nym.com/hc';
export const ContactSupportUrl = 'https://support.nym.com/hc/requests/new';
export const ToSUrl = 'https://nym.com/vpn-terms';
export const PrivacyPolicyUrl = 'https://nym.com/vpn-privacy-statement';
export const LocationDetailsArticle =
  'https://support.nym.com/hc/articles/26448676449297-How-is-server-location-determined-by-NymVPN';
export const ModesDetailsArticle =
  'https://support.nym.com/hc/articles/24326365096721-What-s-the-difference-between-Fast-and-Anonymous-mode';
export const SentryHomePage = 'https://sentry.io/';
export const CountryCacheDuration = 120; // seconds
export const HomeThrottleDelay = 6000;
