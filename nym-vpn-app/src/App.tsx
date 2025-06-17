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
import { RouteLoading, ThemeSetter } from './ui';
import { GatewaysProvider } from './contexts/gateways';
import { IntroAnim } from './screens';

let initialized = false;
const noSplash = window._APP.noSplash;

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

    const showAppWindow = () => {
      console.info('show main window');
      invoke<void>('show_main_window').catch((e: unknown) => console.error(e));
    };
    showAppWindow();
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
    <>
      {!noSplash && <IntroAnim />}
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
    </>
  );
}

export default App;
