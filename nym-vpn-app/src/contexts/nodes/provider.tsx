import { useCallback, useEffect, useState } from 'react';
import { Country, Gateway, GatewaysByCountry } from '../../types';
import { useMainState } from '../main';
import { useLang } from '../../hooks';
import { NodesContext } from './context';
import { UiGateway, UiGatewaysByCountry } from './types';
import { isSelectedNodeType } from './util';

export type NodesStateProviderProps = {
  children: React.ReactNode;
  nodeType: 'entry' | 'exit';
};

function NodesProvider({ children, nodeType }: NodesStateProviderProps) {
  const {
    vpnMode,
    entryNode,
    exitNode,
    mxEntryGateways,
    mxExitGateways,
    wgGateways,
  } = useMainState();

  const [nodes, setNodes] = useState<UiGatewaysByCountry[]>([]);
  const [loading, setLoading] = useState(true);

  const { compare, getCountryName } = useLang();

  const uifyGateways = useCallback(
    (
      list: GatewaysByCountry[],
      selectedEntry: Country | Gateway,
      selectedExit: Country | Gateway,
    ) => {
      return list
        .map<UiGatewaysByCountry>((country) => {
          const isCountrySelected = isSelectedNodeType(
            country.country,
            selectedEntry,
            selectedExit,
          );
          const gateways = country.gateways.map<UiGateway>((gw) => {
            return {
              ...gw,
              isSelected: isSelectedNodeType(gw, selectedEntry, selectedExit),
            };
          });

          return {
            country: {
              ...country.country,
              isSelected: isCountrySelected,
            },
            type: country.type,
            gateways,
            isSelected: isCountrySelected,
            i18n: getCountryName(country.country.code) || country.country.name,
          };
        })
        .sort((a, b) => compare(a.i18n, b.i18n));
    },
    [compare, getCountryName],
  );

  useEffect(() => {
    setLoading(true);
    if (vpnMode === 'Mixnet' && nodeType === 'entry') {
      setNodes(uifyGateways(mxEntryGateways, entryNode, exitNode));
    } else if (vpnMode === 'Mixnet' && nodeType === 'exit') {
      setNodes(uifyGateways(mxExitGateways, entryNode, exitNode));
    } else {
      setNodes(uifyGateways(wgGateways, entryNode, exitNode));
    }
    setLoading(false);
  }, [
    nodeType,
    entryNode,
    exitNode,
    mxEntryGateways,
    mxExitGateways,
    uifyGateways,
    vpnMode,
    wgGateways,
  ]);

  return (
    <NodesContext.Provider
      value={{
        nodes,
        loading,
      }}
    >
      {children}
    </NodesContext.Provider>
  );
}

export default NodesProvider;
