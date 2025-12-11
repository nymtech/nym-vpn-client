import { memo } from 'react';
import { Accordion } from '@base-ui-components/react';
import {
  SelectedKind,
  SelectedUiNode,
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
  UiRegion,
} from '../../../contexts';
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
}: {
  node: UiGatewaysByCountry;
  hop: NodeHop;
  vpnMode: VpnMode;
  quicFilter: boolean;
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
        gwCount={gateways.length}
      />
      <Accordion.Panel
        data-testid={`country-accordion-content-${country.code}`}
        className="w-full flex flex-col gap-3"
      >
        {country.code.toLowerCase() === 'us' ? (
          regions.map((region) => (
            <Accordion.Item
              className="first:pt-3"
              key={region.name}
              value={region.name}
              render={() => (
                <>
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
                    gwCount={region.gateways.length}
                    sub
                  />
                  <Accordion.Panel>
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
                  </Accordion.Panel>
                </>
              )}
            ></Accordion.Item>
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
      </Accordion.Panel>
    </>
  );
});
