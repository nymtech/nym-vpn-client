import { useCallback, useMemo, useState } from 'react';
import { NodeListStateContext } from './context';

export type NodeListPrevStateProps = {
  children: React.ReactNode;
};

function NodeListPrevStateProvider({ children }: NodeListPrevStateProps) {
  const [entryExpanded, setEntryExpanded] = useState<string[]>([]);
  const [exitExpanded, setExitExpanded] = useState<string[]>([]);
  const [entryFocused, setEntryFocused] = useState<string | null>(null);
  const [exitFocused, setExitFocused] = useState<string | null>(null);

  const setExpanded = useCallback(
    (nodeType: 'entry' | 'exit', value: string[]) => {
      if (nodeType === 'entry') {
        setEntryExpanded(value);
      } else {
        setExitExpanded(value);
      }
    },
    [],
  );

  const setFocused = useCallback((nodeType: 'entry' | 'exit', key: string) => {
    if (nodeType === 'entry') {
      setEntryFocused(key);
    } else {
      setExitFocused(key);
    }
  }, []);

  const reset = useCallback((hop: 'entry' | 'exit' | 'all') => {
    switch (hop) {
      case 'entry':
        setEntryExpanded([]);
        break;
      case 'exit':
        setExitExpanded([]);
        break;
      case 'all':
        setEntryExpanded([]);
        setExitExpanded([]);
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
