import { useCallback, useEffect, useState } from 'react';
import {
  UiGateway,
  UiGatewaysByCountry,
  UiRegion,
  useNodeList,
  useNodeListState,
} from '../../../contexts';
import { NodeHop } from '../../../types';
import { sortByScore } from './util';

export function useFilterList(hop: NodeHop) {
  const { nodes, gateways, vpnMode } = useNodeList();
  const { addToExpanded, setExpanded, entry, exit } = useNodeListState();
  const search = hop === 'entry' ? entry.search : exit.search;

  const [filteredNodes, setFilteredNodes] =
    useState<UiGatewaysByCountry[]>(nodes);
  const [filteredGateways, setFilteredGateways] = useState<UiGateway[]>([]);

  const filter = useCallback(
    (value: string) => {
      if (value.length <= 0) {
        // reset
        setFilteredNodes(nodes);
        setFilteredGateways([]);
        setExpanded(hop, []);
        return;
      }

      const lowCaseValue = value.toLowerCase();
      let usRegions: UiRegion[] = [];
      const filteredNodes = structuredClone(nodes).filter((node) => {
        if (node.country.code.toLowerCase() === 'us') {
          usRegions = node.regions.filter((region) => {
            return region.name.toLowerCase().includes(lowCaseValue);
          });
          if (usRegions.length > 0) {
            addToExpanded(hop, node.country.code);
            return true;
          }
        }
        // toLowerCase() is used to make it case-insensitive
        return node.i18n.toLowerCase().includes(lowCaseValue);
      });
      if (usRegions.length > 0) {
        const index = filteredNodes.findIndex(
          (n) => n.country.code.toLowerCase() === 'us',
        );
        if (index !== -1) {
          filteredNodes[index].regions = usRegions;
        }
      }
      const filteredGw = gateways.filter((gw) => {
        return (
          gw.name.toLowerCase().includes(lowCaseValue) ||
          gw.location.city.toLowerCase().includes(lowCaseValue)
        );
      });
      filteredGw.sort((a, b) => {
        if (vpnMode === 'mixnet') {
          return sortByScore(a.mxScore, b.mxScore);
        } else {
          return sortByScore(a.wgScore, b.wgScore);
        }
      });

      setFilteredNodes(filteredNodes);
      setFilteredGateways(filteredGw);
    },
    [gateways, nodes, vpnMode, addToExpanded, setExpanded, hop],
  );

  // refresh the UI list whenever the backend gateway data changes
  useEffect(() => {
    if (search) {
      filter(search);
    } else {
      setFilteredNodes(nodes);
      setFilteredGateways([]);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [nodes, gateways]);

  return {
    filter,
    nodes: filteredNodes,
    gateways: filteredGateways,
  };
}
