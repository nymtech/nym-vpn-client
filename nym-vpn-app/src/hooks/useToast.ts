import { useCallback } from 'react';
import { Toast, ToastManagerAddOptions } from '@base-ui/react';

export type ToastAddData = ToastManagerAddOptions<object> & {
  type: 'error' | 'warn' | 'info' | 'success' | 'ghost';
};

const useToast = () => {
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

  return { add, close };
};

export default useToast;
