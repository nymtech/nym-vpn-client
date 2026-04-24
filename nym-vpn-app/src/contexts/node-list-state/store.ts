import { create } from 'zustand';
import { useShallow } from 'zustand/react/shallow';
import { Focused } from './types';

export type Hop = 'entry' | 'exit';

type HopState = {
  expanded: string[];
  focused: Focused | null;
  search: string | null;
};

type NodeListStore = {
  entry: HopState;
  exit: HopState;
  setExpanded: (hop: Hop, value: string[]) => void;
  addToExpanded: (hop: Hop, value: string) => void;
  setFocused: (hop: Hop, focused: Focused | null) => void;
  setSearch: (hop: Hop, search: string | null) => void;
  reset: (hop: Hop | 'all') => void;
};

const emptyHop: HopState = { expanded: [], focused: null, search: null };

export const useNodeListStateStore = create<NodeListStore>((set, get) => ({
  entry: { ...emptyHop },
  exit: { ...emptyHop },

  setExpanded: (hop, value) =>
    set((s) => ({ [hop]: { ...s[hop], expanded: value } })),

  addToExpanded: (hop, value) => {
    if (get()[hop].expanded.includes(value)) return;
    set((s) => ({ [hop]: { ...s[hop], expanded: [...s[hop].expanded, value] } }));
  },

  setFocused: (hop, focused) =>
    set((s) => ({ [hop]: { ...s[hop], focused } })),

  setSearch: (hop, search) =>
    set((s) => ({ [hop]: { ...s[hop], search } })),

  reset: (hop) => {
    if (hop === 'all') {
      set({ entry: { ...emptyHop }, exit: { ...emptyHop } });
    } else {
      set({ [hop]: { ...emptyHop } });
    }
  },
}));

export const useNodeListState = () => useNodeListStateStore(useShallow((s) => s));
