import { createContext, useContext } from 'react';

type HopState = {
  // list of country items which are expanded
  // country 2-letter codes
  expanded: string[];
};

type State = {
  entry: HopState;
  exit: HopState;
  setExpanded: (
    nodeType: 'entry' | 'exit',
    // country codes
    value: string[],
  ) => void;
  reset: (hop: 'entry' | 'exit' | 'all') => void;
};

const initialState: State = {
  entry: { expanded: [] },
  exit: { expanded: [] },
  setExpanded: () => {
    /*  SCARECROW */
  },
  reset: () => {
    /*  SCARECROW */
  },
};

export const NodeListStateContext = createContext<State>(initialState);
export const useNodeListState = () => {
  return useContext(NodeListStateContext);
};
