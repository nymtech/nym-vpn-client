import { useCallback, useEffect, useMemo, useState } from 'react';
import { Country, Gateway, GatewaysByCountry, NodeHop } from '../../types';
import { useMainState } from '../main';
import { useLang } from '../../hooks';
import { NodesContext } from './context';
import { GwSelectedKind, UiGateway, UiGatewaysByCountry } from './types';
import { isSelectedNodeType } from './util';

export type NodesStateProviderProps = {
  children: React.ReactNode;
  nodeType: NodeHop;
};

function NodesProvider({ children, nodeType }: NodesStateProviderProps) {
  const {
    vpnMode,
    entryNode,
    exitNode,
    mxEntryGateways,
    mxExitGateways,
    wgGateways,
    mxEntryGatewaysError,
    mxExitGatewaysError,
    wgGatewaysError,
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
              isSelected: isSelectedNodeType(
                gw,
                selectedEntry,
                selectedExit,
              ) as GwSelectedKind,
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
    if (vpnMode === 'mixnet' && nodeType === 'entry') {
      list = uifyGateways(mxEntryGateways, entryNode, exitNode);
    } else if (vpnMode === 'mixnet' && nodeType === 'exit') {
      list = uifyGateways(mxExitGateways, entryNode, exitNode);
    } else {
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

  const error = useMemo(() => {
    if (vpnMode === 'mixnet' && nodeType === 'entry') {
      return mxEntryGatewaysError;
    }
    if (vpnMode === 'mixnet' && nodeType === 'exit') {
      return mxExitGatewaysError;
    }
    return wgGatewaysError;
  }, [
    mxEntryGatewaysError,
    mxExitGatewaysError,
    nodeType,
    vpnMode,
    wgGatewaysError,
  ]);

  return (
    <NodesContext.Provider
      value={{
        nodes,
        gateways: gatewayList,
        loading,
        node: nodeType,
        vpnMode,
        error,
      }}
    >
      {children}
    </NodesContext.Provider>
  );
}

export default NodesProvider;
