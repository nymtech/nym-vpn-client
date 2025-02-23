import { createContext, useContext } from 'react';
import { UiGateway, UiGatewaysByCountry } from './types';

type NodesState = {
  nodes: UiGatewaysByCountry[];
  gateways: UiGateway[];
  loading: boolean;
};

const initialState: NodesState = {
  nodes: [],
  gateways: [],
  loading: false,
};

export const NodesContext = createContext<NodesState>(initialState);
export const useNodesState = () => {
  return useContext(NodesContext);
};
