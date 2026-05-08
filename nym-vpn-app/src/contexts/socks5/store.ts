import { useShallow } from 'zustand/react/shallow';
import { useAppStore } from '../../store';

// Expose the combined store under the original name for backward compatibility.
export { useAppStore as useSocks5Store } from '../../store';

export const useSocks5 = () =>
  useAppStore(
    useShallow((s) => ({
      status: s.status,
      isLoading: s.isLoading,
      enable: s.enable,
      disable: s.disable,
      refresh: s.refresh,
    })),
  );
