import { useCallback, useMemo, useState } from 'react';
import { Hop, NodeListStateContext } from './context';
import { Focused } from './types';

export type NodeListPrevStateProps = {
  children: React.ReactNode;
};

function NodeListPrevStateProvider({ children }: NodeListPrevStateProps) {
  const [entryExpanded, setEntryExpanded] = useState<string[]>([]);
  const [exitExpanded, setExitExpanded] = useState<string[]>([]);
  const [entryFocused, setEntryFocused] = useState<Focused | null>(null);
  const [exitFocused, setExitFocused] = useState<Focused | null>(null);
  const [entrySearch, setEntrySearch] = useState<string | null>(null);
  const [exitSearch, setExitSearch] = useState<string | null>(null);

  const setExpanded = useCallback((hop: Hop, value: string[]) => {
    if (hop === 'entry') {
      setEntryExpanded(value);
    } else {
      setExitExpanded(value);
    }
  }, []);

  const addToExpanded = useCallback(
    (hop: Hop, value: string) => {
      const expanded = hop === 'entry' ? entryExpanded : exitExpanded;
      if (expanded.includes(value)) {
        return;
      }
      if (hop === 'entry') {
        setEntryExpanded([...entryExpanded, value]);
      } else {
        setExitExpanded([...exitExpanded, value]);
      }
    },
    [entryExpanded, exitExpanded],
  );

  const setFocused = useCallback((hop: Hop, focus: Focused) => {
    if (hop === 'entry') {
      setEntryFocused(focus);
    } else {
      setExitFocused(focus);
    }
  }, []);

  const setSearch = useCallback((hop: Hop, search: string | null) => {
    if (hop === 'entry') {
      setEntrySearch(search);
    } else {
      setExitSearch(search);
    }
  }, []);

  const reset = useCallback((hop: Hop | 'all') => {
    switch (hop) {
      case 'entry':
        setEntryExpanded([]);
        setEntryFocused(null);
        setEntrySearch(null);
        break;
      case 'exit':
        setExitExpanded([]);
        setExitFocused(null);
        setExitSearch(null);
        break;
      case 'all':
        setEntryExpanded([]);
        setEntryFocused(null);
        setEntrySearch(null);
        setExitExpanded([]);
        setExitFocused(null);
        setExitSearch(null);
        break;
    }
  }, []);

  const ctx = useMemo(
    () => ({
      entry: {
        expanded: entryExpanded,
        focused: entryFocused,
        search: entrySearch,
      },
      exit: {
        expanded: exitExpanded,
        focused: exitFocused,
        search: exitSearch,
      },
      setExpanded,
      setFocused,
      setSearch,
      addToExpanded,
      reset,
    }),
    [
      entryExpanded,
      entryFocused,
      entrySearch,
      exitExpanded,
      exitFocused,
      exitSearch,
      setExpanded,
      setFocused,
      setSearch,
      addToExpanded,
      reset,
    ],
  );

  return (
    <NodeListStateContext.Provider value={ctx}>
      {children}
    </NodeListStateContext.Provider>
  );
}

export default NodeListPrevStateProvider;
