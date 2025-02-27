import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router';
import {
  Country,
  DbKey,
  Gateway,
  GatewaysByCountry,
  NodeHop,
  StateDispatch,
  isGateway,
} from '../../types';
import { routes } from '../../router';
import { useMainDispatch, useMainState } from '../main';
import { useLang } from '../../hooks';
import { kvSet } from '../../kvStore';
import { NodesContext } from './context';
import {
  GwSelectedKind,
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
} from './types';
import { isSelectedNodeType, uiNodeToRaw } from './util';

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
  const dispatch = useMainDispatch() as StateDispatch;

  const [nodes, setNodes] = useState<UiGatewaysByCountry[]>([]);
  const [gatewayList, setGatewayList] = useState<UiGateway[]>([]);
  const [loading, setLoading] = useState(true);

  const { compare, getCountryName } = useLang();
  const navigate = useNavigate();

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

  const onNodeSelect = useCallback(
    async (node: NodeHop, selected: UiCountry | UiGateway) => {
      if (
        isGateway(selected) &&
        (selected.isSelected === 'exit' || selected.isSelected === 'entry')
      ) {
        return;
      }

      let key: DbKey;
      if (node === 'entry') {
        key = vpnMode === 'wg' ? 'wg-entry-node' : 'mx-entry-node';
      } else {
        key = vpnMode === 'wg' ? 'wg-exit-node' : 'mx-exit-node';
      }

      try {
        await kvSet(key, uiNodeToRaw(selected));
        dispatch({
          type: 'set-node',
          payload: { hop: node, node: selected },
        });
      } catch (e) {
        console.warn(e);
      }
      navigate(routes.root);
    },
    [dispatch, navigate, vpnMode],
  );

  return (
    <NodesContext.Provider
      value={{
        nodes,
        gateways: gatewayList,
        loading,
        node: nodeType,
        vpnMode,
        error,
        onNodeSelect,
      }}
    >
      {children}
    </NodesContext.Provider>
  );
}

export default NodesProvider;
