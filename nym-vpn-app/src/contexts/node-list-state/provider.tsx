import { useCallback, useMemo, useState } from 'react';
import { NodeListStateContext } from './context';

export type NodeListPrevStateProps = {
  children: React.ReactNode;
};

function NodeListPrevStateProvider({ children }: NodeListPrevStateProps) {
  const [entryExpanded, setEntryExpanded] = useState<string[]>([]);
  const [exitExpanded, setExitExpanded] = useState<string[]>([]);

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
      entry: { expanded: entryExpanded },
      exit: { expanded: exitExpanded },
      setExpanded,
      reset,
    }),
    [entryExpanded, exitExpanded, reset, setExpanded],
  );

  return (
    <NodeListStateContext.Provider value={ctx}>
      {children}
    </NodeListStateContext.Provider>
  );
}

export default NodeListPrevStateProvider;
