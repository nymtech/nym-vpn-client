import { createContext, useContext } from 'react';
import { MixnetTrafficConfig } from '../../../../types';
import { MixnetConfigState } from './reducer';

export type MixnetTrafficConfigContextType = {
  state: MixnetConfigState;
  hasUnsavedSettings: boolean;
  hasSettingsOtherThanDefaults: boolean;
  updateField: (
    field: keyof MixnetTrafficConfig,
    value: number | boolean,
  ) => void;
  restoreDefaults: () => void;
  continuousItems: { value: number; label: string }[];
  backgroundCoverItems: { value: number; label: string }[];
  mixingDelay: { minValue: number; maxValue: number; defaultValue: number };
};

export const MixnetTrafficConfigContext =
  createContext<MixnetTrafficConfigContextType | null>(null);

export function useMixnetTrafficConfig() {
  const context = useContext(MixnetTrafficConfigContext);
  if (!context) {
    throw new Error(
      'useMixnetTrafficConfig must be used within MixnetTrafficConfigProvider',
    );
  }
  return context;
}
