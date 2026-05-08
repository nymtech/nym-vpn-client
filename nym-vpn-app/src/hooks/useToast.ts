import { useCallback } from 'react';
import { Toast, ToastManagerAddOptions } from '@base-ui/react';

export type ToastAddData = ToastManagerAddOptions<object> & {
  type: 'error' | 'warn' | 'info' | 'success' | 'ghost';
};

const useToast = () => {
  const { add: addToast, close: closeToast } = Toast.useToastManager();

  const add = useCallback(
    (data: ToastAddData) => {
      return addToast(data as ToastManagerAddOptions<object>);
    },
    [addToast],
  );

  const close = useCallback(
    (id: string) => {
      closeToast(id);
    },
    [closeToast],
  );

  return { add, close };
};

export default useToast;
