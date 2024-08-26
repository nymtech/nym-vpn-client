// entry point for the error window

import React from 'react';
import ReactDOM from 'react-dom/client';
import clsx from 'clsx';
import { invoke } from '@tauri-apps/api';
import { exit } from '@tauri-apps/api/process';
import { Button, MsIcon } from './ui';
import { StartupError, StartupErrorKey } from './types';

function getErrorText(key: StartupErrorKey) {
  switch (key) {
    case 'StartupOpenDb':
      return 'Failed to open the application database.';
    case 'StartupOpenDbLocked':
      return 'It is likely that the application is already running. You cannot run multiple instances of the application at the same time.';
    default:
      return 'Unknown error';
  }
}

(async () => {
  const error = await invoke<StartupError | undefined>('startup_error');

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <div
        className={clsx([
          'flex flex-col items-center justify-between h-full p-8 gap-10 cursor-default select-none',
        ])}
      >
        <div className="flex flex-col mt-auto">
          <div className="flex flex-row items-center gap-2">
            <MsIcon
              className="text-2xl font-bold text-cement-feet"
              icon={'error'}
            />
            <h1 className="text-xl font-semibold tracking-wider leading-loose">
              Oops!
            </h1>
          </div>
          <p className="font-semibold">
            Something went wrong while loading the app.
          </p>
          {error && <p className="font-semibold">{getErrorText(error?.key)}</p>}
          {error?.details && (
            <div className="overflow-auto overscroll-auto max-h-32 mt-4">
              <p
                id="error-content"
                className="italic text-mercury-mist cursor-auto select-text"
              >
                {error.details}
              </p>
            </div>
          )}
        </div>
        <Button
          color="cornflower"
          onClick={() => {
            exit(0);
          }}
          className="max-w-48 mt-auto"
        >
          Close
        </Button>
      </div>
    </React.StrictMode>,
  );
})();
