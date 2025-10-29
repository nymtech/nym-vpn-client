import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Country,
  Gateway,
  GatewaysByCountry,
  NodeHop,
  SelectedNode,
} from '../../types';
import { useMainState } from '../main';
import { useGateways } from '../gateways';
import { useLang } from '../../hooks';
import { NodeListContext } from './context';
import {
  GwSelectedKind,
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
  UiRegion,
} from './types';
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

  const countryToUi: (
    country: Country,
    selectedEntry: SelectedNode,
    selectedExit: SelectedNode,
  ) => UiCountry = useCallback(
    (
      country: Country,
      selectedEntry: SelectedNode,
      selectedExit: SelectedNode,
    ) => {
      const isCountrySelected = isSelectedNodeType(
        country,
        selectedEntry,
        selectedExit,
      );
      return {
        ...country,
        nodeType: 'country',
        isSelected: isCountrySelected,
      };
    },
    [],
  );

  const gatewaysToUi = useCallback(
    (
      gateways: Gateway[],
      selectedEntry: SelectedNode,
      selectedExit: SelectedNode,
    ) => {
      return gateways.reduce<UiGateway[]>((gwAcc, gw) => {
        if (quicFilter && !gw.quic) {
          return gwAcc;
        }
        const uiGw: UiGateway = {
          ...gw,
          nodeType: 'gateway',
          isSelected: isSelectedNodeType(
            gw,
            selectedEntry,
            selectedExit,
          ) as GwSelectedKind,
        };
        gwAcc.push(uiGw);
        return gwAcc;
      }, []);
    },
    [quicFilter],
  );

  const uifyGateways = useCallback(
    (
      list: GatewaysByCountry[],
      selectedEntry: SelectedNode,
      selectedExit: SelectedNode,
    ) => {
      return list
        .reduce<UiGatewaysByCountry[]>((countryAcc, country) => {
          if (quicFilter && !country.quic) {
            return countryAcc;
          }
          const mappedCountry = countryToUi(
            country.country,
            selectedEntry,
            selectedExit,
          );
          const gateways = gatewaysToUi(
            country.gateways,
            selectedEntry,
            selectedExit,
          );
          if (gateways.length === 0) {
            return countryAcc;
          }

          const regions = country.regions.reduce<UiRegion[]>((acc, region) => {
            const regionGateways = gatewaysToUi(
              region.gateways,
              selectedEntry,
              selectedExit,
            );
            if (regionGateways.length === 0) {
              return acc;
            }
            acc.push({
              ...region,
              nodeType: 'region',
              gateways: regionGateways,
              isSelected: isSelectedNodeType(
                region,
                selectedEntry,
                selectedExit,
              ) as GwSelectedKind,
            });
            return acc;
          }, []);

          const uiCountry: UiGatewaysByCountry = {
            country: mappedCountry,
            regions,
            gateways,
            type: country.type,
            isSelected: mappedCountry.isSelected,
            i18n: getCountryName(country.country.code) || country.country.name,
          };
          countryAcc.push(uiCountry);
          return countryAcc;
        }, [])
        .sort((a, b) => compare(a.i18n, b.i18n));
    },
    [quicFilter, countryToUi, gatewaysToUi, getCountryName, compare],
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
