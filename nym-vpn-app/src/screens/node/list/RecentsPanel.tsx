import { useTranslation } from 'react-i18next';
import { AppError, NodeHop, VpnMode } from '../../../types';
import { SelectedUiNode, UiGateway } from '../../../types/node';
import ListLoading from './ListLoading';
import RecentsEmpty from './RecentsEmpty';
import RecentsList from './RecentsList';

export type RecentsPanelProps = {
  /** Every recent gateway for this hop, before the search term is applied. */
  gateways: UiGateway[];
  /** The subset left after the search term, in daemon recency order. */
  searched: UiGateway[];
  loading: boolean;
  error: AppError | null;
  onSelect: (node: SelectedUiNode) => void;
  onNodeDetails: (node: UiGateway) => void;
  hop: NodeHop;
  vpnMode: VpnMode;
  quicFilter: boolean;
};

/**
 * The recents view of the node list.
 *
 * Recents comes from its own daemon lookup rather than the country tree, so this
 * panel owns its loading, error and empty states. An error stays inside it:
 * unlike a gateway list failure, a failed recents lookup must not take the
 * screen down.
 *
 * Order matters below. Anything already fetched wins, so a search matching
 * nothing falls through to `RecentsList`'s no-results copy rather than reading as
 * "no recents". And loading is checked before the empty state, so a user who
 * *has* recents never sees the empty copy flash on the way in.
 */
function RecentsPanel({
  gateways,
  searched,
  loading,
  error,
  onSelect,
  onNodeDetails,
  hop,
  vpnMode,
  quicFilter,
}: RecentsPanelProps) {
  const { t } = useTranslation('node-location');

  if (gateways.length > 0) {
    return (
      <RecentsList
        gateways={searched}
        onSelect={onSelect}
        onNodeDetails={onNodeDetails}
        hop={hop}
        vpnMode={vpnMode}
        quicFilter={quicFilter}
      />
    );
  }

  if (loading) return <ListLoading />;

  if (error) {
    return (
      <p className="text-status-error px-6 py-4" data-testid="recents-error">
        {t('recents.error')}
      </p>
    );
  }

  return <RecentsEmpty />;
}

export default RecentsPanel;
