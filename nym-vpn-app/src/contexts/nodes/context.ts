import { createContext, useContext } from 'react';
import { NodeHop, VpnMode } from '../../types';
import { UiGateway, UiGatewaysByCountry } from './types';

type NodesState = {
  nodes: UiGatewaysByCountry[];
  gateways: UiGateway[];
  loading: boolean;
  node: NodeHop;
  vpnMode: VpnMode;
};

const initialState: NodesState = {
  nodes: [],
  gateways: [],
  loading: false,
  node: 'entry',
  vpnMode: 'TwoHop',
};

export const NodesContext = createContext<NodesState>(initialState);
export const useNodesState = () => {
  return useContext(NodesContext);
};
