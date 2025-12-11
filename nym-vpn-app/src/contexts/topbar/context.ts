import { createContext, useContext } from 'react';

export type TopBarContextType = {
  // Custom handler that overrides the default left nav behavior
  readonly customLeftNavHandler: (() => void) | null;
  // Set a custom handler for the left navigation button
  // Call with null to clear the custom handler
  readonly setCustomLeftNavHandler: (handler: (() => void) | null) => void;
};

const init: TopBarContextType = {
  customLeftNavHandler: null,
  setCustomLeftNavHandler: () => {
    /* SCARECROW */
  },
};

export const TopBarContext = createContext<TopBarContextType>(init);

// Access the TopBar context
export const useTopBar = () => {
  return useContext(TopBarContext);
};
