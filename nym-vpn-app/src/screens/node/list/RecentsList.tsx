import { memo } from 'react';
import { dequal } from 'dequal';
import { Trans, useTranslation } from 'react-i18next';
import { NodeHop, VpnMode } from '../../../types';
import { SelectedUiNode, UiGateway } from '../../../types/node';
import { Link } from '../../../ui';
import { ContactSupportUrl, DocsUrl } from '../../../constants';
import GatewayItem from './GatewayItem';
import { PanelContent } from './NodeListPanelContent';

export type RecentsListProps = {
  gateways: UiGateway[];
  onSelect: (node: SelectedUiNode) => void;
  onNodeDetails: (node: UiGateway) => void;
  hop: NodeHop;
  vpnMode: VpnMode;
  quicFilter: boolean;
};

/**
 * Flat list of the most recently connected gateways, in the order the daemon
 * reports them (most recent first).
 *
 * Not grouped by country, unlike the all/favorites views: grouping destroys the
 * recency ordering that gives the list its meaning. Rows therefore render their
 * full location, since no country header supplies that context.
 */
const RecentsList = memo(function RecentsList({
  gateways,
  onSelect,
  onNodeDetails,
  hop,
  vpnMode,
  quicFilter,
}: RecentsListProps) {
  const { t } = useTranslation('node-location');

  if (gateways.length === 0) {
    return (
      <div className="space-y-4 px-6 py-4" data-testid="recents-no-results">
        <p className="text-text-primary truncate">
          {t('no-results-found.title')}
        </p>
        <p className="text-text-secondary whitespace-pre-line">
          <Trans
            i18nKey="no-results-found.description"
            ns="node-location"
            components={{
              1: <Link url={ContactSupportUrl} color="primary" />,
              2: <Link url={DocsUrl} color="primary" />,
            }}
          />
        </p>
      </div>
    );
  }

  return (
    <div className="pt-2" data-testid="recents-list">
      {gateways.map((gateway) => (
        <PanelContent animate key={gateway.id}>
          <GatewayItem
            node={hop}
            gateway={gateway}
            onSelect={onSelect}
            onNodeDetails={onNodeDetails}
            vpnMode={vpnMode}
            quicLabel={quicFilter}
            fullLocation
          />
        </PanelContent>
      ))}
    </div>
  );
}, arePropsEqual);

export default RecentsList;

function arePropsEqual(
  oldProps: RecentsListProps,
  newProps: RecentsListProps,
): boolean {
  if (oldProps.hop !== newProps.hop) return false;
  if (oldProps.vpnMode !== newProps.vpnMode) return false;
  if (oldProps.quicFilter !== newProps.quicFilter) return false;
  if (oldProps.gateways.length !== newProps.gateways.length) return false;
  return dequal(oldProps.gateways, newProps.gateways);
}
