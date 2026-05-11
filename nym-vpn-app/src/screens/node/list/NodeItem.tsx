import { memo } from 'react';
import { Collapsible } from '@base-ui-components/react';
import {
  SelectedKind,
  SelectedUiNode,
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
  UiRegion,
} from '../../../types/node';
import { NodeHop, VpnMode } from '../../../types';
import GatewayItem from './GatewayItem';
import RowHeader from './RowHeader';
import { PanelContent } from './NodeListPanelContent';

export const NodeItem = memo(function NodeItem({
  node,
  hop,
  vpnMode,
  quicFilter,
  handleLocationSelect,
  onGatewaySelect,
  onNodeDetails,
  expanded,
  onExpandChange,
}: {
  node: UiGatewaysByCountry;
  hop: NodeHop;
  vpnMode: VpnMode;
  quicFilter: boolean;
  expanded: string[];
  onExpandChange: (key: string, open: boolean) => void;
  handleLocationSelect: (
    location: UiCountry | UiRegion,
    isSelected: SelectedKind,
    gwCount: number,
  ) => void;
  onGatewaySelect: (node: SelectedUiNode) => void;
  onNodeDetails: (gateway: UiGateway) => void;
}) {
  const { i18n, isSelected, gateways, country, regions } = node;

  return (
    <>
      <RowHeader
        hop={hop}
        isSelected={isSelected}
        node={country}
        i18n={i18n}
        onClick={() =>
          handleLocationSelect(country, isSelected, gateways.length)
        }
        open={expanded.includes(country.code) || false}
      />
      <Collapsible.Panel
        keepMounted={false}
        data-testid={`country-accordion-content-${country.code}`}
        className="collapsible-panel flex w-full flex-col"
      >
        {country.code.toLowerCase() === 'us' ? (
          regions.map((region) => (
            <Collapsible.Root
              className="group/region border-divider border-b first:pt-0 last:border-b-0"
              key={region.name}
              open={expanded.includes(region.name)}
              onOpenChange={(open) => onExpandChange(region.name, open)}
            >
              <RowHeader
                hop={hop}
                isSelected={region.isSelected}
                node={region}
                i18n={i18n}
                onClick={() => {
                  handleLocationSelect(
                    region,
                    region.isSelected,
                    region.gateways.length,
                  );
                }}
                sub
                open={expanded.includes(region.name)}
              />
              <Collapsible.Panel
                keepMounted={false}
                className="collapsible-panel"
              >
                <PanelContent>
                  {region.gateways.map((gateway) => (
                    <GatewayItem
                      key={gateway.id}
                      node={hop}
                      gateway={gateway}
                      vpnMode={vpnMode}
                      quicLabel={quicFilter}
                      onSelect={onGatewaySelect}
                      onNodeDetails={onNodeDetails}
                    />
                  ))}
                </PanelContent>
              </Collapsible.Panel>
            </Collapsible.Root>
          ))
        ) : (
          <PanelContent>
            {gateways.map((gateway) => (
              <GatewayItem
                key={gateway.id}
                node={hop}
                gateway={gateway}
                onSelect={onGatewaySelect}
                onNodeDetails={onNodeDetails}
                vpnMode={vpnMode}
                quicLabel={quicFilter}
              />
            ))}
          </PanelContent>
        )}
      </Collapsible.Panel>
    </>
  );
});
