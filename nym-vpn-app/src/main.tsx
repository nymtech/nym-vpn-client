import React from 'react';
import ReactDOM from 'react-dom/client';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import duration from 'dayjs/plugin/duration';
import App from './App';
import { mockTauriIPC } from './dev/setup';
import { kvGet } from './kvStore';
import initSentry from './sentry';

// needed locales to load for dayjs
import 'dayjs/locale/es';
import 'dayjs/locale/fr';
import 'dayjs/locale/hi';
import 'dayjs/locale/it';
import 'dayjs/locale/pt-br.js';
import 'dayjs/locale/ru';
import 'dayjs/locale/tr';
import 'dayjs/locale/uk';
import 'dayjs/locale/zh-cn';

if (import.meta.env.MODE === 'dev-browser') {
  console.log('Running in dev-browser mode. Mocking tauri window and IPCs');
  mockTauriIPC();
}

dayjs.extend(relativeTime);
dayjs.extend(duration);

(async () => {
  const monitoring = await kvGet<boolean>('Monitoring');

  if (monitoring) {
    await initSentry();
  }

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
})();
