import { invoke } from '@tauri-apps/api';
import { useEffect } from 'react';
import { RouterProvider } from 'react-router-dom';
import dayjs from 'dayjs';
import { useTranslation } from 'react-i18next';
import router from './router';
import { sleep } from './helpers';
import { MainStateProvider } from './state';
import './i18n/config';
import { ThemeSetter } from './ui';

function App() {
  const { i18n } = useTranslation();
  dayjs.locale(i18n.language);

  useEffect(() => {
    const showAppWindow = async () => {
      // allow more time to the app window to be fully ready
      // avoiding the initial "white flash"
      await sleep(100);
      invoke<void>('show_main_window')
        .then(() => {
          console.log('show_main_window invoked');
          const splashLogo = document.getElementById('splash-logo');
          if (splashLogo) {
            // show the nym logo in the splash-screen
            splashLogo.style.opacity = '100';
          }
        })
        .catch((e) => console.error(e));
    };
    showAppWindow();
  }, []);

  return (
    <MainStateProvider>
      <ThemeSetter>
        <RouterProvider router={router} />
      </ThemeSetter>
    </MainStateProvider>
  );
}

export default App;
