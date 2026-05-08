import { useShallow } from 'zustand/react/shallow';
import { useAppStore } from '../../store';
import type { GatewaysState } from './types';

export const useGateways = (): GatewaysState =>
  useAppStore(
    useShallow((s) => ({
      mxEntry: s.mxEntry,
      mxExit: s.mxExit,
      wg: s.wg,
      mxEntryLoading: s.mxEntryLoading,
      mxExitLoading: s.mxExitLoading,
      wgLoading: s.wgLoading,
      mxEntryError: s.mxEntryError,
      mxExitError: s.mxExitError,
      wgError: s.wgError,
    })),
  );

export const useFetchGateways = () => useAppStore((s) => s.fetchGateways);

export const useLookupGw = () => useAppStore((s) => s.lookupGw);
