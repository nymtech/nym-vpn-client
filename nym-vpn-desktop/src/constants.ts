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
  'https://github.com/nymtech/nym-vpn-client/issues';
export const MatrixRoomUrl = 'https://matrix.to/#/%23NymVPN:nymtech.chat';
export const DiscordInviteUrl = 'https://discord.com/invite/nym';
export const FaqUrl = 'https://nymvpn.com/en/support';
export const EmailSupportUrl = 'mailto:support@nymvpn.com';
export const ToSUrl = 'https://nymvpn.com/en/terms';
export const PrivacyPolicyUrl = 'https://nymvpn.com/en/privacy?type=apps';
export const CountryCacheDuration = 120000; // 2 minutes
