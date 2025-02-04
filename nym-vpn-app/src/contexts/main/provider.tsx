import { invoke } from '@tauri-apps/api/core';
import React, { useCallback, useEffect, useReducer } from 'react';
import { useTranslation } from 'react-i18next';
import { CountryCacheDuration } from '../../constants';
import {
  MainDispatchContext,
  MainStateContext,
  useInAppNotify,
} from '../index';
import { sleep } from '../../util';
import { kvSet } from '../../kvStore';
import {
  BackendError,
  Cli,
  Country,
  NodeHop,
  SystemMessage,
  VpnMode,
} from '../../types';
import { initFirstBatch, initSecondBatch } from '../../state/init';
import { initialState, reducer } from '../../state';
import { useTauriEvents } from '../../state/useTauriEvents';
import { S_STATE } from '../../static';
import { MCache } from '../../cache';

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
    networkEnv,
  } = state;

  const { push } = useInAppNotify();
  useTauriEvents(dispatch, push);

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
    initFirstBatch(dispatch, push).then(async () => {
      console.log('init of 1st batch done');
      dispatch({ type: 'init-done' });
      const args = await invoke<Cli>(`cli_args`);
      // skip the animation if NOSPLASH is set
      if (import.meta.env.APP_NOSPLASH || args.nosplash) {
        return;
      }
      // wait for the splash screen to be visible for a short time
      // as init phase is very fast
      // duration → 700ms
      await sleep(700);
      const splash = document.getElementById('splash');
      if (splash) {
        // starts the fade out animation
        splash.style.opacity = '0';
        // fade out animation duration is set to 150ms, so we wait 300ms
        // to ensure it's done before removing the splash screen
        await sleep(300);
        splash.remove();
        console.log('splash animation done');
      }
    });

    // this second batch is not needed for the app to be fully
    // functional, and continue loading in the background
    initSecondBatch(dispatch).then(() => {
      console.log('init of 2nd batch done');
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // whenever the vpn mode changes, refresh the countries or use cached ones
  useEffect(() => {
    if (!S_STATE.vpnModeInit) {
      return;
    }
    if (vpnMode === 'Mixnet') {
      fetchCountries(vpnMode, 'entry');
      fetchCountries(vpnMode, 'exit');
    } else {
      fetchCountries(vpnMode, 'entry');
    }
  }, [vpnMode]);

  // whenever the network environment changes (e.i. daemon has been reconfigured),
  // clear cache
  useEffect(() => {
    if (!S_STATE.networkEnvInit) {
      return;
    }
    console.info(`network env changed ${networkEnv}, clearing cache`);
    MCache.clear();
  }, [networkEnv]);

  useEffect(() => {
    if (S_STATE.systemMessageInit) {
      return;
    }
    S_STATE.systemMessageInit = true;
    const querySystemMessages = async () => {
      try {
        const messages = await invoke<SystemMessage[]>('system_messages');
        if (messages.length > 0) {
          console.info('system messages', messages);
          push({
            text: messages
              .map(({ name, message }) => `${name}: ${message}`)
              .join('\n'),
            position: 'top',
            closeIcon: true,
            autoHideDuration: 10000,
          });
        }
      } catch (e) {
        console.warn('failed to query system messages:', e);
      }
    };
    querySystemMessages();
  }, [push]);

  // use cached values if any, otherwise query from daemon
  const fetchCountries = async (vpnMode: VpnMode, node: NodeHop) => {
    // first try to load from cache
    let countries = MCache.get<Country[]>(
      vpnMode === 'Mixnet' ? `mn-${node}-countries` : 'wg-countries',
    );
    // fallback to daemon query
    if (!countries) {
      console.info(`fetching countries for ${vpnMode} ${node}`);
      try {
        countries = await invoke<Country[]>('get_countries', {
          vpnMode,
          nodeType: node === 'entry' ? 'Entry' : 'Exit',
        });
        MCache.set(
          vpnMode === 'Mixnet' ? `mn-${node}-countries` : 'wg-countries',
          countries,
          CountryCacheDuration,
        );
      } catch (e) {
        console.warn(`Failed to fetch ${node} countries:`, e);
        dispatch({
          type:
            node === 'entry'
              ? 'set-entry-countries-error'
              : 'set-exit-countries-error',
          payload: e as BackendError,
        });
        if (vpnMode === 'TwoHop') {
          // in 2hop mode, the error must be set for both entry and exit
          dispatch({
            type:
              node === 'entry'
                ? 'set-exit-countries-error'
                : 'set-entry-countries-error',
            payload: e as BackendError,
          });
        }
      }
    }
    if (!countries) {
      console.warn('no countries found');
      return;
    }
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
  };

  const checkSelectedCountry = useCallback(
    async (hop: NodeHop, countries: Country[], selected: Country) => {
      if (
        countries.length > 0 &&
        !countries.some((c) => c.code === selected.code)
      ) {
        const location =
          countries[Math.floor(Math.random() * countries.length)];
        console.info(
          `selected ${hop} country [${selected.code}] not available, switching to [${location.code}]`,
        );
        try {
          await kvSet<Country>(
            hop === 'entry' ? 'EntryNodeLocation' : 'ExitNodeLocation',
            location,
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

  const fetchMnCountries = useCallback(
    async (node: NodeHop) => fetchCountries(vpnMode, node),
    [vpnMode],
  );

  const fetchWgCountries = useCallback(
    async () => fetchCountries(vpnMode, 'entry'),
    [vpnMode],
  );

  return (
    <MainStateContext.Provider
      value={{
        ...state,
        fetchMnCountries,
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
