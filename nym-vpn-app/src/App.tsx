import { Suspense, useEffect } from 'react';
import { RouterProvider } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { type } from '@tauri-apps/plugin-os';
import * as Toast from '@radix-ui/react-toast';
import dayjs from 'dayjs';
import customParseFormat from 'dayjs/plugin/customParseFormat';
import { useTranslation } from 'react-i18next';
import {
  DialogProvider,
  InAppNotificationProvider,
  MainStateProvider,
  NodeListStateProvider,
  Socks5Provider,
  TopBarProvider,
} from './contexts';
import { useLang } from './hooks';
import { LngTag } from './i18n';
import { kvGet } from './kvStore';
import router from './router';
import './i18n/config';
import { RouteLoading, ThemeSetter } from './ui';
import { GatewaysProvider } from './contexts/gateways';
import { IntroAnim, IntroSplash } from './screens';
import { InitState } from './types';
import { PrivyProvider } from './PrivyProvider';

let initialized = false;
const noSplash = window._APP.noSplash;
const os = type();

function App({ init }: { init: InitState }) {
  const { i18n } = useTranslation();
  dayjs.locale(i18n.language);
  dayjs.extend(customParseFormat);

  const { set } = useLang();
  const intro =
    os === 'windows' ? (
      <IntroAnim theme={init.uiTheme} />
    ) : (
      <IntroSplash theme={init.uiTheme} />
    );

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
      {!noSplash && intro}
      <InAppNotificationProvider>
        <Toast.Provider>
          <MainStateProvider init={init}>
            <PrivyProvider>
              <GatewaysProvider>
                <NodeListStateProvider>
                  <Socks5Provider>
                    <ThemeSetter>
                      <DialogProvider>
                        <TopBarProvider>
                          <Suspense fallback={<RouteLoading />}>
                            <RouterProvider router={router} />
                          </Suspense>
                        </TopBarProvider>
                      </DialogProvider>
                    </ThemeSetter>
                  </Socks5Provider>
                </NodeListStateProvider>
              </GatewaysProvider>
            </PrivyProvider>
          </MainStateProvider>
        </Toast.Provider>
      </InAppNotificationProvider>
    </>
  );
}

export default App;
