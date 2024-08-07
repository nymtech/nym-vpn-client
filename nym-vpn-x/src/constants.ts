import { Country } from './types';

export const AppName = 'NymVPN';
export const ConnectionEvent = 'connection-state';
export const ProgressEvent = 'connection-progress';
export const DaemonEvent = 'vpnd-status';
// TODO ⚠ keep this value in sync with the one declared in `index.html`
export const DefaultRootFontSize = 14; // in px
export const DefaultNodeCountry: Country = {
  name: 'France',
  code: 'FR',
};
// TODO disabled Fastest location until the backend is ready
export const FastestFeatureEnabled = false;
export const DefaultThemeMode = 'System';

// Various external links
export const GitHubIssuesUrl =
  'https://www.nymtech.net/go/github/nym-vpn-client/issues';
export const MatrixRoomUrl = 'https://matrix.to/#/%23NymVPN:nymtech.chat';
export const DiscordInviteUrl = 'https://nymtech.net/go/discord';
export const FaqUrl = 'https://support.nymvpn.com/hc/en-us';
export const ContactSupportUrl =
  'https://support.nymvpn.com/hc/en-us/requests/new';
export const ToSUrl = 'https://nymvpn.com/en/terms';
export const PrivacyPolicyUrl = 'https://nymvpn.com/en/privacy?type=apps';
export const LocationDetailsArticle =
  'https://support.nymvpn.com/hc/en-us/articles/26448676449297-How-is-server-location-determined-by-NymVPN';
export const SentryHomePage = 'https://sentry.io/';
export const CountryCacheDuration = 120000; // 2 minutes
export const HomeThrottleDelay = 6000;
