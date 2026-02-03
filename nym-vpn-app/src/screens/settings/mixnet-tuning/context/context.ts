import { createContext, useContext } from 'react';
import { MixnetTrafficConfig } from '../../../../types';
import { MixnetTrafficConfigAction } from './reducer';

export type MixnetTrafficConfigContextType = {
  state: MixnetTrafficConfig;
  dispatch: React.Dispatch<MixnetTrafficConfigAction>;
  hasUnsavedSettings: boolean;
  hasSettingsOtherThanDefaults: boolean;
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
