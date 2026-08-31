import { memo } from 'react';
import { dequal } from 'dequal';
import { Collapsible } from '@base-ui-components/react';
import { Trans, useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { NodeHop, VpnMode } from '../../../types';
import { Focused, useNodeListState } from '../../../store/nodeListState';
import {
  SelectedKind,
  SelectedUiNode,
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
  UiRegion,
} from '../../../types/node';
import { Link } from '../../../ui';
import { ContactSupportUrl, DocsUrl } from '../../../constants';
import { NodeItem } from './NodeItem';
import GatewayItem from './GatewayItem';
import { PanelContent } from './NodeListPanelContent';

export type NodeListProps = {
  nodes: UiGatewaysByCountry[];
  gateways: UiGateway[];
  onSelect: (node: SelectedUiNode) => void;
  onNodeDetails: (node: UiGateway) => void;
  hop: NodeHop;
  vpnMode: VpnMode;
  quicFilter: boolean;
  expanded: string[];
  focused: Focused | null;
};

const NodeList = memo(function NodeList({
  nodes,
  gateways,
  onSelect,
  hop,
  vpnMode,
  quicFilter,
  onNodeDetails,
  expanded,
}: NodeListProps) {
  const { setExpanded } = useNodeListState();
  const { t } = useTranslation('node-location');

  const handleLocationSelect = (
    location: UiCountry | UiRegion,
    isSelected: SelectedKind,
    gwCount: number,
  ) => {
    if (isSelected && isSelected !== hop && gwCount <= 1) {
      return;
    }
    if (isSelected !== hop && isSelected !== 'entry-and-exit') {
      onSelect(location);
    }
  };

  const onExpandChange = (key: string, open: boolean) => {
    const next = open ? [...expanded, key] : expanded.filter((k) => k !== key);
    setExpanded(hop, next);
  };

  if (nodes.length === 0 && gateways.length === 0) {
    return (
      <div className="space-y-4 px-6 py-4">
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
    <div className="mr-0">
      <div
        className="flex w-full flex-col gap-3 p-3"
        data-testid="node-list-accordion"
      >
        {nodes.map((node) => (
          <Collapsible.Root
            key={node.country.code}
            open={expanded.includes(node.country.code)}
            onOpenChange={(open) => onExpandChange(node.country.code, open)}
          >
            <NodeItem
              key={node.country.code}
              node={node}
              hop={hop}
              vpnMode={vpnMode}
              quicFilter={quicFilter}
              handleLocationSelect={handleLocationSelect}
              onGatewaySelect={onSelect}
              onNodeDetails={onNodeDetails}
              expanded={expanded}
              onExpandChange={onExpandChange}
            />
          </Collapsible.Root>
        ))}
      </div>
      {gateways.length > 0 && (
        <div className="mt-2" data-testid="standalone-gateways-container">
          <h3
            className={clsx('text-text-secondary truncate px-4', {
              'py-6': nodes.length > 0,
            })}
          >
            {t('search-other-nodes')}
          </h3>
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
      )}
    </div>
  );
}, arePropsEqual);

export default NodeList;

function arePropsEqual(
  oldProps: NodeListProps,
  newProps: NodeListProps,
): boolean {
  if (oldProps.hop !== newProps.hop) return false;
  if (oldProps.vpnMode !== newProps.vpnMode) return false;
  if (oldProps.quicFilter !== newProps.quicFilter) return false;
  if (oldProps.gateways.length !== newProps.gateways.length) return false;
  if (oldProps.nodes.length !== newProps.nodes.length) return false;
  if (!dequal(oldProps.expanded, newProps.expanded)) return false;
  if (!dequal(oldProps.focused, newProps.focused)) return false;
  if (!dequal(oldProps.gateways, newProps.gateways)) return false;
  if (!dequal(oldProps.nodes, newProps.nodes)) return false;
  return true;
}
