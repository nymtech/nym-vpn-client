import { create } from 'zustand';
import { useShallow } from 'zustand/react/shallow';

export type Hop = 'entry' | 'exit';

export type Focused = {
  type: 'gateway' | 'region' | 'country';
  // country 2-letter code | region name | gateway ID
  key: string;
};

/** Which subset of the node list is shown. */
export type ListView = 'all' | 'favorites' | 'recents';

type HopState = {
  expanded: string[];
  focused: Focused | null;
  search: string | null;
  view: ListView;
};

type NodeListStore = {
  entry: HopState;
  exit: HopState;
  setExpanded: (hop: Hop, value: string[]) => void;
  addToExpanded: (hop: Hop, value: string) => void;
  setFocused: (hop: Hop, focused: Focused | null) => void;
  setSearch: (hop: Hop, search: string | null) => void;
  setView: (hop: Hop, view: ListView) => void;
  reset: (hop: Hop | 'all') => void;
};

const emptyHop: HopState = {
  expanded: [],
  focused: null,
  search: null,
  view: 'all',
};

export const useNodeListStateStore = create<NodeListStore>((set, get) => ({
  entry: { ...emptyHop },
  exit: { ...emptyHop },

  setExpanded: (hop, value) =>
    set((s) => ({ [hop]: { ...s[hop], expanded: value } })),

  addToExpanded: (hop, value) => {
    if (get()[hop].expanded.includes(value)) return;
    set((s) => ({
      [hop]: { ...s[hop], expanded: [...s[hop].expanded, value] },
    }));
  },

  setFocused: (hop, focused) => set((s) => ({ [hop]: { ...s[hop], focused } })),

  setSearch: (hop, search) => set((s) => ({ [hop]: { ...s[hop], search } })),

  setView: (hop, view) => set((s) => ({ [hop]: { ...s[hop], view } })),

  // `view` deliberately survives a reset: expanded/focused/search are list
  // positioning that should be cleared on navigation, whereas the active view is
  // a choice the user made for the session.
  reset: (hop) => {
    if (hop === 'all') {
      set((s) => ({
        entry: { ...emptyHop, view: s.entry.view },
        exit: { ...emptyHop, view: s.exit.view },
      }));
    } else {
      set((s) => ({ [hop]: { ...emptyHop, view: s[hop].view } }));
    }
  },
}));

export const useNodeListState = () =>
  useNodeListStateStore(useShallow((s) => s));
