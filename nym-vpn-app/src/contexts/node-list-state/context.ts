import { createContext, useContext } from 'react';
import { Focused } from './types';

type HopState = {
  // list of country items which are expanded,
  // country 2-letter codes
  expanded: string[];
  // last node focused in the list
  focused: Focused | null;
};

type State = {
  entry: HopState;
  exit: HopState;
  setExpanded: (
    hop: 'entry' | 'exit',
    // country codes
    value: string[],
  ) => void;
  setFocused: (hop: 'entry' | 'exit', focused: Focused) => void;
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
