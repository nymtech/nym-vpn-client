// entry point for the error window

import React from 'react';
import ReactDOM from 'react-dom/client';
import clsx from 'clsx';
import { invoke } from '@tauri-apps/api';
import { exit } from '@tauri-apps/api/process';
import { Button, MsIcon } from './ui';

(async () => {
  const error = await invoke<string | undefined>('startup_error');

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
          {error && (
            <div className="overflow-auto overscroll-auto max-h-32 mt-4">
              <p
                id="error-content"
                className="italic text-mercury-mist cursor-auto select-text"
              >
                {error}
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
