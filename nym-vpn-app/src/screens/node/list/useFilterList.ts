import { useCallback, useEffect, useState } from 'react';
import { useDebouncedCallback as useDebounce } from 'use-debounce';
import {
  UiGateway,
  UiGatewaysByCountry,
  UiRegion,
  useNodeList,
} from '../../../contexts';

export function useFilterList() {
  const { nodes, gateways } = useNodeList();

  const [filteredNodes, setFilteredNodes] =
    useState<UiGatewaysByCountry[]>(nodes);
  const [filteredGateways, setFilteredGateways] =
    useState<UiGateway[]>(gateways);

  // refresh the UI list whenever the backend gateway data changes
  useEffect(() => {
    setFilteredNodes(nodes);
    setFilteredGateways([]);
  }, [nodes, gateways]);

  const filter = useCallback(
    (value: string) => {
      if (value.length <= 0) {
        // reset
        setFilteredNodes(nodes);
        setFilteredGateways([]);
        return;
      }

      let usRegions: UiRegion[] = [];
      const filteredNodes = structuredClone(nodes).filter((node) => {
        if (node.country.code.toLowerCase() === 'us') {
          usRegions = node.regions.filter((region) => {
            return region.name.toLowerCase().includes(value.toLowerCase());
          });
          if (usRegions.length > 0) {
            return true;
          }
        }
        // toLowerCase() is used to make it case-insensitive
        return node.i18n.toLowerCase().includes(value.toLowerCase());
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
        return gw.name.toLowerCase().includes(value.toLowerCase());
      });
      setFilteredNodes(filteredNodes);
      setFilteredGateways(filteredGw);
    },
    [gateways, nodes],
  );

  const debounced = useDebounce((value: string) => {
    filter(value);
  }, 200);

  return {
    filter: debounced,
    nodes: filteredNodes,
    gateways: filteredGateways,
  };
}
