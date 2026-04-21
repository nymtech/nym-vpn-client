import { Toast, ToastManagerAddOptions } from '@base-ui/react';
import { useCallback, useMemo } from 'react';
import { NewToastContext, ToastAddData } from './context';

function NewToastProvider({ children }: { children: React.ReactNode }) {
  const toastManager = Toast.useToastManager();

  const add = useCallback(
    (data: ToastAddData) => {
      return toastManager.add(data as ToastManagerAddOptions<object>);
    },
    [toastManager],
  );

  const close = useCallback(
    (id: string) => {
      toastManager.close(id);
    },
    [toastManager],
  );

  const ctx = useMemo(() => ({ add, close }), [add, close]);
  return (
    <NewToastContext.Provider value={ctx}>{children}</NewToastContext.Provider>
  );
}

function NewToastProviderWrapper({ children }: { children: React.ReactNode }) {
  return (
    <Toast.Provider timeout={100000}>
      <NewToastProvider>{children}</NewToastProvider>
    </Toast.Provider>
  );
}

export default NewToastProviderWrapper;
