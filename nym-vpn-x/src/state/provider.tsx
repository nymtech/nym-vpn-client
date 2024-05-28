import { invoke } from '@tauri-apps/api';
import React, { useCallback, useEffect, useReducer } from 'react';
import { CountryCacheDuration } from '../constants';
import { MainDispatchContext, MainStateContext } from '../contexts';
import { sleep } from '../helpers';
import { useThrottle } from '../hooks';
import { BackendError, Cli, Country, NodeHop, isCountry } from '../types';
import { initFirstBatch, initSecondBatch } from './init';
import { initialState, reducer } from './main';
import { useTauriEvents } from './useTauriEvents';

type Props = {
  children?: React.ReactNode;
};

export function MainStateProvider({ children }: Props) {
  const [state, dispatch] = useReducer(reducer, initialState);
  const {
    entryCountryList,
    exitCountryList,
    entryNodeLocation,
    exitNodeLocation,
  } = state;

  useTauriEvents(dispatch, state);

  // initialize app state
  useEffect(() => {
    // this first batch is needed to ensure the app is fully
    // initialized and ready, once done splash screen is removed
    // and the UI is shown
    initFirstBatch(dispatch).then(async () => {
      console.log('init of 1st batch done');
      dispatch({ type: 'init-done' });
      const args = await invoke<Cli>(`cli_args`);
      // skip the animation if NOSPLASH is set
      if (import.meta.env.APP_NOSPLASH || args?.nosplash) {
        return;
      }
      // wait for the splash screen to be visible for a short time as
      // init phase is very fast, avoiding flashing the splash screen
      // note: the real duration of splashscreen is this value minus the one
      // declared in `App.tsx`, that is 700 - 100 → 600ms
      await sleep(700);
      const splash = document.getElementById('splash');
      if (splash) {
        // starts the fade out animation
        splash.style.opacity = '0';
        // fade out animation duration is set to 150ms, so we wait 300ms
        // to ensure it's done before removing the splash screen
        await sleep(300);
        splash.remove();
      }
    });

    // this second batch is not needed for the app to be fully
    // functional, and continue loading in the background
    initSecondBatch(dispatch).then(() => {
      console.log('init of 2nd batch done');
    });
  }, []);

  const fetchEntryCountries = useThrottle(
    async () => fetchCountries('entry'),
    CountryCacheDuration,
  );

  const fetchExitCountries = useThrottle(
    async () => fetchCountries('exit'),
    CountryCacheDuration,
  );

  const fetchCountries = useCallback(async (node: NodeHop) => {
    try {
      const countries = await invoke<Country[]>('get_countries', {
        nodeType: node === 'entry' ? 'Entry' : 'Exit',
      });
      dispatch({
        type: 'set-country-list',
        payload: {
          hop: node,
          countries,
        },
      });
      // reset any previous error
      dispatch({
        type:
          node === 'entry'
            ? 'set-entry-countries-error'
            : 'set-exit-countries-error',
        payload: null,
      });
    } catch (e) {
      console.warn(`Failed to fetch ${node} countries:`, e);
      dispatch({
        type:
          node === 'entry'
            ? 'set-entry-countries-error'
            : 'set-exit-countries-error',
        payload: e as BackendError,
      });
    }
  }, []);

  const checkSelectedCountry = useCallback(
    async (hop: NodeHop) => {
      const selected = hop === 'entry' ? entryNodeLocation : exitNodeLocation;
      const countries = hop === 'entry' ? entryCountryList : exitCountryList;
      if (
        countries.length > 0 &&
        isCountry(selected) &&
        !countries.some((c) => c.code === selected.code)
      ) {
        console.warn(
          `selected ${hop} country [${selected.name}] not in the list, picking a random one`,
        );
        const location =
          countries[Math.floor(Math.random() * countries.length)];
        try {
          await invoke<void>('set_node_location', {
            nodeType: hop === 'entry' ? 'Entry' : 'Exit',
            location: isCountry(location) ? { Country: location } : 'Fastest',
          });
          dispatch({
            type: 'set-node-location',
            payload: { hop, location },
          });
        } catch (e) {
          console.warn(`failed to update the selected country: ${e}`);
        }
      }
    },
    [entryNodeLocation, exitNodeLocation, entryCountryList, exitCountryList],
  );

  useEffect(() => {
    // if the current country is not in the list of available countries, pick a random one
    if (entryCountryList.length > 0) {
      checkSelectedCountry('entry');
    }
    if (exitCountryList.length > 0) {
      checkSelectedCountry('exit');
    }
  }, [
    checkSelectedCountry,
    entryNodeLocation,
    exitNodeLocation,
    entryCountryList,
    exitCountryList,
  ]);

  return (
    <MainStateContext.Provider
      value={{ ...state, fetchEntryCountries, fetchExitCountries }}
    >
      <MainDispatchContext.Provider value={dispatch}>
        {children}
      </MainDispatchContext.Provider>
    </MainStateContext.Provider>
  );
}
