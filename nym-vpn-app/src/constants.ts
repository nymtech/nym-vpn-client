import { SafestNode, SelectedNode } from './types';

// declared in src-tauri/src/events.rs
export const AppName = 'NymVPN';
export const TunnelStateEvent = 'tunnel-state';
export const AccountStateEvent = 'account-state';
export const DaemonEvent = 'vpnd-status';
export const MixnetEvent = 'mixnet-event';
export const VpnConfigEvent = 'vpn-config';
export const UpdatePendingEvent = 'update-pending';
export const DiagnosticsSuggestedEvent = 'diagnostics-suggested';
// ⚠ keep this value in sync with the one declared in `index.html`
export const DefaultRootFontSize = 14; // in px

export const DefaultNode: SelectedNode = SafestNode;
export const DefaultThemeMode = 'system';
// ⚠ keep those in sync with the theme definition in `styles.css`
export const ColorMainBgLight = '#242b2d';
export const ColorMainBgDark = '#ebeef4';

// Various external links
export const DownloadAppUrl = 'https://nym.com/download';
export const GitHubIssuesUrl =
  'https://nym.com/go/github/nym-vpn-client/issues';
export const MatrixRoomUrl = 'https://nym.com/go/matrix';
export const TelegramUrl = 'https://nym.com/go/telegram';
export const DiscordInviteUrl = 'https://nym.com/go/discord';
export const FaqUrl = 'https://support.nym.com/hc';
export const ContactSupportUrl = 'https://support.nym.com/hc/requests/new';
export const TranslationHelpUrl = 'https://crowdin.com/editor/nymvpn-apps';
export const ToSUrl = 'https://nym.com/vpn-terms';
export const PrivacyPolicyUrl = 'https://nym.com/vpn-privacy-statement';
export const LocationDetailsArticle =
  'https://support.nym.com/hc/articles/26448676449297-How-is-server-location-determined-by-NymVPN';
export const ModesDetailsArticle =
  'https://support.nym.com/hc/articles/24326365096721-What-s-the-difference-between-Fast-and-Anonymous-mode';
export const SentryHomePage = 'https://sentry.io/';
export const GatewaysCacheDuration = 120; // 2min
export const NymVpnPricingUrl = 'https://nym.com/pricing';
export const NymVpnAccountLoginUrl = 'https://nym.com/account/login';
export const SentryPrivacyPolicyUrl = 'https://sentry.io/privacy/';
export const AnonNetworkStatsUrl = 'https://nym.com/anonymous-stats';
export const QuicUrl = 'https://nym.com/features/quic';
export const DomainFrontingUrl = 'https://nym.com/features/stealth-api-connect';
export const AmneziaWgUrl =
  'https://support.nym.com/hc/en-us/articles/28104383231121-How-does-NymVPN-implement-Wireguard';
export const IpInfoIoUrl = 'https://ipinfo.io';
export const SupportServerLocationUrl =
  'https://support.nym.com/hc/en-us/articles/26448676449297-How-is-server-location-determined-by-NymVPN';
export const NetworkExplorerNodeUrl = 'https://nym.com/explorer/nym-node';
export const countriesWithRegions = ['US', 'CA', 'AU', 'MX', 'BR', 'IN', 'CN'];
export const ResidentialIpServersUrl =
  'https://support.nym.com/hc/en-us/articles/35279486714641-Why-can-t-I-access-streaming-services-while-using-NymVPN';
export const QuicSupportArticleUrl =
  'https://support.nym.com/hc/en-us/articles/39648047741457-QUIC-transport-mode';
export const LocationAccuracyLink =
  'https://support.nym.com/hc/en-us/articles/26448676449297-How-is-server-location-determined-by-NymVPN';
export const CustomDnsHelpUrl = 'https://nym.com/features/custom-dns';
export const MixnetParametersLearnMoreUrl =
  'https://nym.com/features/mixnet-tuning';
export const DocsUrl = 'https://nym.com/docs';
