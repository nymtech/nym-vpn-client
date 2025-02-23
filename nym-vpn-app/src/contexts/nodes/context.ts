import { createContext, useContext } from 'react';
import { UiGatewaysByCountry } from './types';

type NodesState = {
  nodes: UiGatewaysByCountry[];
  loading: boolean;
};

const initialState: NodesState = {
  nodes: [],
  loading: false,
};

export const NodesContext = createContext<NodesState>(initialState);
export const useNodesState = () => {
  return useContext(NodesContext);
};
