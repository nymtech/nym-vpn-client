import { invoke } from '@tauri-apps/api/core';
import React, { useCallback, useEffect, useReducer } from 'react';
import { useTranslation } from 'react-i18next';
import { CountryCacheDuration } from '../../constants';
import {
  MainDispatchContext,
  MainStateContext,
  useInAppNotify,
} from '../index';
import { sleep } from '../../helpers';
import { useThrottle } from '../../hooks';
import { kvSet } from '../../kvStore';
import {
  BackendError,
  Cli,
  Country,
  NodeHop,
  NodeLocation,
  isCountry,
} from '../../types';
import { initFirstBatch, initSecondBatch } from '../../state/init';
import { initialState, reducer } from '../../state';
import { useTauriEvents } from '../../state/useTauriEvents';
import { S_STATE } from '../../static';

let initialized = false;

type Props = {
  children?: React.ReactNode;
};

function MainStateProvider({ children }: Props) {
  const [state, dispatch] = useReducer(reducer, initialState);
  const {
    entryCountryList,
    exitCountryList,
    entryNodeLocation,
    exitNodeLocation,
    vpnMode,
  } = state;

  useTauriEvents(dispatch, state);
  const { push } = useInAppNotify();

  const { t } = useTranslation();

  // initialize app state
  useEffect(() => {
    if (initialized) {
      return;
    }
    initialized = true;

    // this first batch is needed to ensure the app is fully
    // initialized and ready, once done splash screen is removed
    // and the UI is shown
    initFirstBatch(dispatch).then(async () => {
      console.log('init of 1st batch done');
      dispatch({ type: 'init-done' });
      const args = await invoke<Cli>(`cli_args`);
      // skip the animation if NOSPLASH is set
      if (import.meta.env.APP_NOSPLASH || args.nosplash) {
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

  useEffect(() => {
    if (!S_STATE.vpnModeInit) {
      return;
    }
    if (vpnMode === 'Mixnet') {
      fetchCountries('entry');
      fetchCountries('exit');
    } else {
      fetchCountries('entry');
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [vpnMode]);

  const fetchMxEntryCountries = useThrottle(
    async () => fetchCountries('entry'),
    CountryCacheDuration,
    [vpnMode],
  );

  const fetchMxExitCountries = useThrottle(
    async () => fetchCountries('exit'),
    CountryCacheDuration,
    [vpnMode],
  );

  const fetchWgCountries = useThrottle(
    // does not matter if entry or exit, the list is the same
    async () => fetchCountries('entry'),
    CountryCacheDuration,
    [vpnMode],
  );

  const fetchCountries = useCallback(
    async (node: NodeHop) => {
      try {
        const countries = await invoke<Country[]>('get_countries', {
          vpnMode,
          nodeType: node === 'entry' ? 'Entry' : 'Exit',
        });
        if (vpnMode === 'Mixnet') {
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
        } else {
          // in 2hop mode, the country list is the same for both entry and exit
          dispatch({
            type: 'set-fast-country-list',
            payload: {
              countries,
            },
          });
          dispatch({
            type: 'set-entry-countries-error',
            payload: null,
          });
          dispatch({
            type: 'set-exit-countries-error',
            payload: null,
          });
        }
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
    },
    [vpnMode],
  );

  const checkSelectedCountry = useCallback(
    async (hop: NodeHop, countries: Country[], selected: NodeLocation) => {
      if (
        countries.length > 0 &&
        isCountry(selected) &&
        !countries.some((c) => c.code === selected.code)
      ) {
        console.info(
          `selected ${hop} country [${selected.name}] not in the list, picking a random one`,
        );
        const location =
          countries[Math.floor(Math.random() * countries.length)];
        try {
          await kvSet<NodeLocation>(
            hop === 'entry' ? 'EntryNodeLocation' : 'ExitNodeLocation',
            isCountry(location) ? location : 'Fastest',
          );
          dispatch({
            type: 'set-node-location',
            payload: { hop, location },
          });
          push({
            text: t(
              hop === 'entry'
                ? 'location-not-available.entry'
                : 'location-not-available.exit',
              {
                ns: 'nodeLocation',
                location: location.name,
              },
            ),
            position: 'top',
            closeIcon: true,
            autoHideDuration: 10000,
          });
        } catch (e) {
          console.warn(`failed to update the selected country: ${e}`);
        }
      }
    },
    [push, t],
  );

  useEffect(() => {
    // if the current country is not in the list of available countries, pick a random one
    if (entryCountryList.length > 0) {
      checkSelectedCountry('entry', entryCountryList, entryNodeLocation);
    }
    if (exitCountryList.length > 0) {
      checkSelectedCountry('exit', exitCountryList, exitNodeLocation);
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
      value={{
        ...state,
        fetchMxEntryCountries,
        fetchMxExitCountries,
        fetchWgCountries,
      }}
    >
      <MainDispatchContext.Provider value={dispatch}>
        {children}
      </MainDispatchContext.Provider>
    </MainStateContext.Provider>
  );
}

export default MainStateProvider;
