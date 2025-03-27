import { Country } from './types';

export const AppName = 'NymVPN';
// ⚠ keep this default in sync with the one declared in
// src-tauri/src/state/app.rs
export const DefaultVpnMode = 'wg';
export const TunnelStateEvent = 'tunnel-state';
export const ProgressEvent = 'connection-progress';
export const DaemonEvent = 'vpnd-status';
export const MixnetEvent = 'mixnet-event';
// ⚠ keep this value in sync with the one declared in `index.html`
export const DefaultRootFontSize = 14; // in px
// NOTE: when fresh country data is get from daemon, the selected countries
// are checked against it and if needed it is automatically switched to
// available ones
export const DefaultCountry: Country = {
  name: 'Switzerland',
  code: 'CH',
};
export const DefaultThemeMode = 'system';
// ⚠ keep those in sync with the theme definition in `styles.css`
export const ColorMainBgLight = '#242b2d';
export const ColorMainBgDark = '#ebeef4';

// Various external links
export const DownloadAppUrl = 'https://nym.com/download';
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
export const GatewaysCacheDuration = 120; // seconds
export const NymVpnPricingUrl = 'https://nym.com/pricing';
