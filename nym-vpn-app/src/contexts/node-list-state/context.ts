import { createContext, useContext } from 'react';
import { Focused } from './types';

export type Hop = 'entry' | 'exit';

type HopState = {
  // list of country items which are expanded,
  // country 2-letter codes
  expanded: string[];
  // last node focused in the list
  focused: Focused | null;
  search: string | null;
};

type State = {
  entry: HopState;
  exit: HopState;
  setExpanded: (
    hop: Hop,
    // country codes
    value: string[],
  ) => void;
  addToExpanded: (
    hop: Hop,
    // country code
    value: string,
  ) => void;
  setFocused: (hop: Hop, focused: Focused | null) => void;
  setSearch: (hop: Hop, search: string | null) => void;
  reset: (hop: Hop | 'all') => void;
};

const initialState: State = {
  entry: { expanded: [], focused: null, search: null },
  exit: { expanded: [], focused: null, search: null },
  setExpanded: () => {
    /*  SCARECROW */
  },
  addToExpanded: () => {
    /*  SCARECROW */
  },
  setFocused: () => {
    /*  SCARECROW */
  },
  setSearch: () => {
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
