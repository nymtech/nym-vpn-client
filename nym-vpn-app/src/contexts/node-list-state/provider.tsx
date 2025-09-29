import { useCallback, useMemo, useState } from 'react';
import { NodeListStateContext } from './context';
import { Focused } from './types';

export type NodeListPrevStateProps = {
  children: React.ReactNode;
};

function NodeListPrevStateProvider({ children }: NodeListPrevStateProps) {
  const [entryExpanded, setEntryExpanded] = useState<string[]>([]);
  const [exitExpanded, setExitExpanded] = useState<string[]>([]);
  const [entryFocused, setEntryFocused] = useState<Focused | null>(null);
  const [exitFocused, setExitFocused] = useState<Focused | null>(null);

  const setExpanded = useCallback((hop: 'entry' | 'exit', value: string[]) => {
    if (hop === 'entry') {
      setEntryExpanded(value);
    } else {
      setExitExpanded(value);
    }
  }, []);

  const setFocused = useCallback((hop: 'entry' | 'exit', focus: Focused) => {
    if (hop === 'entry') {
      setEntryFocused(focus);
    } else {
      setExitFocused(focus);
    }
  }, []);

  const reset = useCallback((hop: 'entry' | 'exit' | 'all') => {
    switch (hop) {
      case 'entry':
        setEntryExpanded([]);
        setEntryFocused(null);
        break;
      case 'exit':
        setExitExpanded([]);
        setExitFocused(null);
        break;
      case 'all':
        setEntryExpanded([]);
        setEntryFocused(null);
        setExitExpanded([]);
        setExitFocused(null);
    }
  }, []);

  const ctx = useMemo(
    () => ({
      entry: { expanded: entryExpanded, focused: entryFocused },
      exit: { expanded: exitExpanded, focused: exitFocused },
      setExpanded,
      setFocused,
      reset,
    }),
    [
      entryExpanded,
      exitExpanded,
      entryFocused,
      exitFocused,
      setExpanded,
      setFocused,
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
