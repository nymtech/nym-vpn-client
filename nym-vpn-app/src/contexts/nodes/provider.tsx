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
  const [gatewayList, setGatewayList] = useState<UiGateway[]>([]);
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

  const toGatewayList = useCallback(
    (list: UiGatewaysByCountry[]) => {
      return (
        list
          .reduce<UiGateway[]>((acc, cur) => {
            return [...acc, ...cur.gateways];
          }, [])
          // TODO instead sort by score?
          .sort((a, b) => compare(a.name, b.name))
      );
    },
    [compare],
  );

  useEffect(() => {
    setLoading(true);
    let list = [];
    if (vpnMode === 'Mixnet' && nodeType === 'entry') {
      console.log('___rendering list for mx-entry');
      list = uifyGateways(mxEntryGateways, entryNode, exitNode);
    } else if (vpnMode === 'Mixnet' && nodeType === 'exit') {
      console.log('___rendering list for mx-exit');
      list = uifyGateways(mxExitGateways, entryNode, exitNode);
    } else {
      console.log('___rendering list for wg');
      list = uifyGateways(wgGateways, entryNode, exitNode);
    }
    setNodes(list);
    setGatewayList(toGatewayList(list));
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
    toGatewayList,
  ]);

  return (
    <NodesContext.Provider
      value={{
        nodes,
        gateways: gatewayList,
        loading,
      }}
    >
      {children}
    </NodesContext.Provider>
  );
}

export default NodesProvider;
