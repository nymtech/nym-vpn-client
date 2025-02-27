import { createContext, useContext } from 'react';
import { AppError, NodeHop, VpnMode } from '../../types';
import { UiCountry, UiGateway, UiGatewaysByCountry } from './types';

type NodesState = {
  nodes: UiGatewaysByCountry[];
  gateways: UiGateway[];
  loading: boolean;
  node: NodeHop;
  vpnMode: VpnMode;
  error?: AppError | null;
  onNodeSelect: (
    node: NodeHop,
    selected: UiCountry | UiGateway,
  ) => Promise<void>;
};

const initialState: NodesState = {
  nodes: [],
  gateways: [],
  loading: false,
  node: 'entry',
  vpnMode: 'wg',
  error: null,
  onNodeSelect: async () => {
    /* SCARECROW */
  },
};

export const NodesContext = createContext<NodesState>(initialState);
export const useNodesState = () => {
  return useContext(NodesContext);
};
