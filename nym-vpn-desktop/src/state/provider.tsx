import React, { useEffect, useReducer } from 'react';
import { MainDispatchContext, MainStateContext } from '../contexts';
import init from './init';
import { initialState, reducer } from './main';
import { useTauriEvents } from './useTauriEvents';

type Props = {
  children?: React.ReactNode;
};

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function MainStateProvider({ children }: Props) {
  const [state, dispatch] = useReducer(reducer, initialState);

  useTauriEvents(dispatch);

  // initialize app state
  useEffect(() => {
    init(dispatch).then(async () => {
      dispatch({ type: 'init-done' });
      await sleep(600);
      const splash = document.getElementById('splash');
      if (splash) {
        splash.remove();
      }
    });
  }, []);

  return (
    <MainStateContext.Provider value={state}>
      <MainDispatchContext.Provider value={dispatch}>
        {children}
      </MainDispatchContext.Provider>
    </MainStateContext.Provider>
  );
}
