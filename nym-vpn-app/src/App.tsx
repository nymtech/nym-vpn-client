import { useEffect } from 'react';
import { RouterProvider } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import dayjs from 'dayjs';
import customParseFormat from 'dayjs/plugin/customParseFormat';
import { useTranslation } from 'react-i18next';
import { Toast } from '@base-ui/react';
import {
  AutologinProvider,
  DialogProvider,
  GwIndependenceWarningProvider,
  MainStateProvider,
  TopBarProvider,
  TrayProvider,
} from './contexts';
import { describeError } from './errors';
import { useLang } from './hooks';
import { LngTag, detectSystemLocale } from './i18n';
import { kvGet } from './kvStore';
import router from './router';
import './i18n/config';
import { ThemeSetter } from './ui';
import { InitState } from './types';

let initialized = false;

function App({ init }: { init: InitState }) {
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
      const stored = await kvGet<string | undefined>('ui-language');
      if (stored) {
        await set(stored as LngTag, false);
      } else {
        const lng = await detectSystemLocale();
        await set(lng, false);
      }
    };
    setLng().catch((e: unknown) => {
      console.error(`failed to set the UI language: ${describeError(e)}`);
    });
  }, [set]);

  return (
    <Toast.Provider timeout={5000}>
      <MainStateProvider init={init}>
        <TrayProvider>
          <ThemeSetter>
            <DialogProvider>
              <GwIndependenceWarningProvider>
                <TopBarProvider>
                  <AutologinProvider>
                    <RouterProvider router={router} />
                  </AutologinProvider>
                </TopBarProvider>
              </GwIndependenceWarningProvider>
            </DialogProvider>
          </ThemeSetter>
        </TrayProvider>
      </MainStateProvider>
    </Toast.Provider>
  );
}

export default App;
