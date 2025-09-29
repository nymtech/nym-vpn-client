import { createContext, useContext } from 'react';

type HopState = {
  // list of country items which are expanded,
  // country 2-letter codes
  expanded: string[];
  // last node focused in the list,
  // country 2-letter codes | gateway ID
  focused: string | null;
};

type State = {
  entry: HopState;
  exit: HopState;
  setExpanded: (
    nodeType: 'entry' | 'exit',
    // country codes
    value: string[],
  ) => void;
  setFocused: (
    nodeType: 'entry' | 'exit',
    // country 2-letter codes | gateway ID
    key: string,
  ) => void;
  reset: (hop: 'entry' | 'exit' | 'all') => void;
};

const initialState: State = {
  entry: { expanded: [], focused: null },
  exit: { expanded: [], focused: null },
  setExpanded: () => {
    /*  SCARECROW */
  },
  setFocused: () => {
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
