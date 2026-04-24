import { memo } from 'react';
import { dequal } from 'dequal';
import { Accordion } from '@base-ui-components/react';
import { Trans, useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { useShallow } from 'zustand/react/shallow';
import {
  Focused,
  SelectedKind,
  SelectedUiNode,
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
  UiRegion,
  useNodeListState,
} from '../../../contexts';
import { NodeHop, VpnMode } from '../../../types';
import { Link } from '../../../ui';
import { ContactSupportUrl, DocsUrl } from '../../../constants';
import { useAppStore } from '../../../store';
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
  expanded: string[];
  focused: Focused | null;
};

const NodeList = memo(function NodeList({
  nodes,
  gateways,
  onSelect,
  hop,
  vpnMode,
  onNodeDetails,
  expanded,
}: NodeListProps) {
  const { backendFlags, quic } = useAppStore(
    useShallow((s) => ({
      backendFlags: s.backendFlags,
      quic: s.quic,
    })),
  );
  const { setExpanded } = useNodeListState();
  const { t } = useTranslation('node-location');

  const quicFilter =
    vpnMode === 'wg' && hop === 'entry' && backendFlags.quic && quic;

  const handleLocationSelect = (
    location: UiCountry | UiRegion,
    isSelected: SelectedKind,
    gwCount: number,
  ) => {
    if (isSelected && isSelected !== hop && gwCount <= 1) {
      // don't allow selecting a country if it has only one gateway,
      // and it's already selected by the other hop
      return;
    }
    if (isSelected !== hop && isSelected !== 'entry-and-exit') {
      onSelect(location);
    }
  };

  const onValueChange = (value: string[]) => {
    setExpanded(hop, value);
  };

  if (nodes.length === 0 && gateways.length === 0) {
    return (
      <div className="px-6 space-y-4">
        <p className=" text-baltic-sea dark:text-white truncate">
          {t('no-results-found.title')}
        </p>
        <p className="text-iron dark:text-bombay whitespace-pre-line">
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
      <Accordion.Root
        className="w-full flex flex-col gap-3"
        data-testid="node-list-accordion"
        value={expanded}
        onValueChange={onValueChange}
        multiple
      >
        {nodes.map((node) => (
          <Accordion.Item
            key={node.country.code}
            value={node.country.code}
            render={() => (
              <NodeItem
                key={node.country.code}
                node={node}
                hop={hop}
                vpnMode={vpnMode}
                quicFilter={quicFilter}
                handleLocationSelect={handleLocationSelect}
                onGatewaySelect={onSelect}
                onNodeDetails={onNodeDetails}
              />
            )}
          ></Accordion.Item>
        ))}
      </Accordion.Root>
      {gateways.length > 0 && (
        <div className="mt-2" data-testid="standalone-gateways-container">
          <h3
            className={clsx('text-iron dark:text-bombay px-4 truncate', {
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
                inSearchResult
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
  if (oldProps.gateways.length !== newProps.gateways.length) return false;
  if (oldProps.nodes.length !== newProps.nodes.length) return false;
  if (!dequal(oldProps.expanded, newProps.expanded)) return false;
  if (!dequal(oldProps.focused, newProps.focused)) return false;
  if (!dequal(oldProps.gateways, newProps.gateways)) return false;
  if (!dequal(oldProps.nodes, newProps.nodes)) return false;
  return true;
}
