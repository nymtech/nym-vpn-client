import { useCallback, useEffect, useMemo, useState } from 'react';
import { Country, Gateway, GatewaysByCountry, NodeHop } from '../../types';
import { useMainState } from '../main';
import { useGateways } from '../gateways';
import { useLang } from '../../hooks';
import { NodeListContext } from './context';
import { GwSelectedKind, UiGateway, UiGatewaysByCountry } from './types';
import { isSelectedNodeType } from './util';

export type NodesStateProviderProps = {
  children: React.ReactNode;
  hop: NodeHop;
};

function NodeListProvider({ children, hop }: NodesStateProviderProps) {
  const { vpnMode, entryNode, exitNode, quic, backendFlags } = useMainState();
  const {
    mxEntry: mxEntryGateways,
    mxExit: mxExitGateways,
    wg: wgGateways,
    mxEntryLoading,
    mxExitLoading,
    wgLoading,
    mxEntryError,
    mxExitError,
    wgError,
  } = useGateways();

  const [nodes, setNodes] = useState<UiGatewaysByCountry[]>([]);
  const [gatewayList, setGatewayList] = useState<UiGateway[]>([]);
  const quicFilter =
    vpnMode === 'wg' && hop === 'entry' && backendFlags.quic && quic;

  const { compare, getCountryName } = useLang();

  const uifyGateways = useCallback(
    (
      list: GatewaysByCountry[],
      selectedEntry: Country | Gateway,
      selectedExit: Country | Gateway,
    ) => {
      return list
        .reduce<UiGatewaysByCountry[]>((countryAcc, country) => {
          if (quicFilter && !country.quic) {
            return countryAcc;
          }
          const isCountrySelected = isSelectedNodeType(
            country.country,
            selectedEntry,
            selectedExit,
          );
          const gateways = country.gateways.reduce<UiGateway[]>((gwAcc, gw) => {
            if (quicFilter && !gw.quic) {
              return gwAcc;
            }
            const uiGw: UiGateway = {
              ...gw,
              isSelected: isSelectedNodeType(
                gw,
                selectedEntry,
                selectedExit,
              ) as GwSelectedKind,
            };
            gwAcc.push(uiGw);
            return gwAcc;
          }, []);

          const uiCountry: UiGatewaysByCountry = {
            country: {
              ...country.country,
              isSelected: isCountrySelected,
            },
            regions: country.regions,
            type: country.type,
            gateways,
            isSelected: isCountrySelected,
            i18n: getCountryName(country.country.code) || country.country.name,
          };
          countryAcc.push(uiCountry);
          return countryAcc;
        }, [])
        .sort((a, b) => compare(a.i18n, b.i18n));
    },
    [compare, getCountryName, quicFilter],
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
    let list = [];
    if (vpnMode === 'mixnet' && hop === 'entry') {
      list = uifyGateways(mxEntryGateways, entryNode, exitNode);
    } else if (vpnMode === 'mixnet' && hop === 'exit') {
      list = uifyGateways(mxExitGateways, entryNode, exitNode);
    } else {
      list = uifyGateways(wgGateways, entryNode, exitNode);
    }
    setNodes(list);
    setGatewayList(toGatewayList(list));
  }, [
    hop,
    entryNode,
    exitNode,
    mxEntryGateways,
    mxExitGateways,
    uifyGateways,
    vpnMode,
    wgGateways,
    toGatewayList,
  ]);

  const loading = useMemo(() => {
    if (nodes.length > 0) {
      return false;
    }
    if (vpnMode === 'mixnet' && hop === 'entry') {
      return mxEntryLoading;
    }
    if (vpnMode === 'mixnet' && hop === 'exit') {
      return mxExitLoading;
    }
    return wgLoading;
  }, [nodes.length, mxEntryLoading, mxExitLoading, wgLoading, hop, vpnMode]);

  const error = useMemo(() => {
    if (vpnMode === 'mixnet' && hop === 'entry') {
      return mxEntryError;
    }
    if (vpnMode === 'mixnet' && hop === 'exit') {
      return mxExitError;
    }
    return wgError;
  }, [mxEntryError, mxExitError, hop, vpnMode, wgError]);

  const ctx = useMemo(
    () => ({
      nodes,
      gateways: gatewayList,
      loading,
      node: hop,
      vpnMode,
      error,
    }),
    [error, gatewayList, loading, hop, nodes, vpnMode],
  );

  return (
    <NodeListContext.Provider value={ctx}>{children}</NodeListContext.Provider>
  );
}

export default NodeListProvider;
