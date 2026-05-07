import { useCallback, useEffect, useState } from 'react';
import { NodeHop, VpnMode } from '../../../types';
import { UiGateway, UiGatewaysByCountry } from '../../../types/node';
import { useNodeListState } from '../../../store/nodeListState';
import { sortByScore } from './util';

export function useFilterList(
  hop: NodeHop,
  nodes: UiGatewaysByCountry[],
  gateways: UiGateway[],
  vpnMode: VpnMode,
) {
  const { addToExpanded, setExpanded, entry, exit, setFocused } =
    useNodeListState();
  const search = hop === 'entry' ? entry.search : exit.search;

  const [filteredNodes, setFilteredNodes] =
    useState<UiGatewaysByCountry[]>(nodes);
  const [filteredGateways, setFilteredGateways] = useState<UiGateway[]>([]);

  const filter = useCallback(
    (value: string) => {
      if (value.length <= 0) {
        setFilteredNodes(nodes);
        setFilteredGateways([]);
        setExpanded(hop, []);
        setFocused(hop, null);
        return;
      }

      const lowCaseValue = value.toLowerCase();
      const filtered: UiGatewaysByCountry[] = [];
      for (const node of nodes) {
        if (node.country.code.toLowerCase() === 'us') {
          const matchingRegions = node.regions.filter((region) =>
            region.name.toLowerCase().includes(lowCaseValue),
          );
          if (matchingRegions.length > 0) {
            addToExpanded(hop, node.country.code);
            filtered.push({ ...node, regions: matchingRegions });
            continue;
          }
        }
        if (node.i18n.toLowerCase().includes(lowCaseValue)) {
          filtered.push(node);
        }
      }

      const filteredGw = gateways
        .filter(
          (gw) =>
            gw.name.toLowerCase().includes(lowCaseValue) ||
            gw.location.city.toLowerCase().includes(lowCaseValue),
        )
        .sort((a, b) =>
          vpnMode === 'mixnet'
            ? sortByScore(a.mxScore, b.mxScore)
            : sortByScore(a.wgScore, b.wgScore),
        );

      setFilteredNodes(filtered);
      setFilteredGateways(filteredGw);
    },
    [gateways, nodes, vpnMode, addToExpanded, setExpanded, setFocused, hop],
  );

  // Re-apply filter when backend data updates
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
