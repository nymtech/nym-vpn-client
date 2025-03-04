import { Suspense, useEffect } from 'react';
import { RouterProvider } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import * as Toast from '@radix-ui/react-toast';
import dayjs from 'dayjs';
import customParseFormat from 'dayjs/plugin/customParseFormat';
import { useTranslation } from 'react-i18next';
import {
  DialogProvider,
  InAppNotificationProvider,
  MainStateProvider,
} from './contexts';
import { useLang } from './hooks';
import { LngTag } from './i18n';
import { kvGet } from './kvStore';
import router from './router';
import './i18n/config';
import { Cli } from './types';
import { RouteLoading, ThemeSetter } from './ui';
import { GatewaysProvider } from './contexts/gateways';

let initialized = false;

function App() {
  const { i18n } = useTranslation();
  dayjs.locale(i18n.language);
  dayjs.extend(customParseFormat);

  const { set } = useLang();

  useEffect(() => {
    if (initialized) {
      return;
    }
    initialized = true;

    const showSplashAnimation = async () => {
      const args = await invoke<Cli>(`cli_args`);
      // if NOSPLASH is set, skip the splash-screen animation
      if (import.meta.env.APP_NOSPLASH || args.nosplash) {
        console.log('splash-screen disabled');
        const splash = document.getElementById('splash');
        if (splash) {
          splash.remove();
        }
        return;
      }
      console.info('show main window');
      invoke<void>('show_main_window').catch((e: unknown) => console.error(e));
    };
    showSplashAnimation();
  }, []);

  useEffect(() => {
    const setLng = async () => {
      const lng = await kvGet<string | undefined>('ui-language');
      if (lng && i18n.language !== lng) {
        await set(lng as LngTag, false);
      }
    };
    setLng();
  }, [i18n, set]);

  return (
    <InAppNotificationProvider>
      <Toast.Provider>
        <MainStateProvider>
          <GatewaysProvider>
            <ThemeSetter>
              <DialogProvider>
                <Suspense fallback={<RouteLoading />}>
                  <RouterProvider router={router} />
                </Suspense>
              </DialogProvider>
            </ThemeSetter>
          </GatewaysProvider>
        </MainStateProvider>
      </Toast.Provider>
    </InAppNotificationProvider>
  );
}

export default App;
