import { createContext, useContext } from 'react';
import { MixnetTrafficConfigAction, MixnetTrafficConfigState } from './reducer';

export type MixnetTrafficConfigContextType = {
  state: MixnetTrafficConfigState;
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
