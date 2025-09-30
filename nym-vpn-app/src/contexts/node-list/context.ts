import { createContext, useContext } from 'react';
import { AppError, NodeHop, VpnMode } from '../../types';
import { UiGateway, UiGatewaysByCountry } from './types';

type State = {
  nodes: UiGatewaysByCountry[];
  gateways: UiGateway[];
  loading: boolean;
  node: NodeHop;
  vpnMode: VpnMode;
  error?: AppError | null;
};

const initialState: State = {
  nodes: [],
  gateways: [],
  loading: false,
  node: 'entry',
  vpnMode: 'wg',
  error: null,
};

export const NodeListContext = createContext<State>(initialState);
export const useNodeList = () => {
  return useContext(NodeListContext);
};
