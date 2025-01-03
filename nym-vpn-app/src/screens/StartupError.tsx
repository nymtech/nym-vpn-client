import { useEffect } from 'react';
import clsx from 'clsx';
import { exit } from '@tauri-apps/plugin-process';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Button, MsIcon } from '../ui';
import { StartupErrorKey, StartupError as TStartupError } from '../types';

function getErrorText(key: StartupErrorKey) {
  switch (key) {
    case 'StartupOpenDb':
      return 'Failed to open the application database.';
    case 'StartupOpenDbLocked':
      return 'The application is likely already running. Multiple instances cannot be opened simultaneously.';
    default:
      return 'Unknown error';
  }
}

let initialized = false;

function StartupError({
  error,
  theme,
}: {
  error: TStartupError;
  theme: 'light' | 'dark' | null;
}) {
  useEffect(() => {
    if (initialized) {
      return;
    }
    initialized = true;
    const window = getCurrentWindow();
    console.info(`show window [${window.label}]`);
    window.show().catch((e: unknown) => {
      console.error(`failed to show error window: ${e}`);
    });
  }, []);

  return (
    <div className={clsx([theme === 'dark' && 'dark', 'h-full'])}>
      <div
        className={clsx([
          'min-w-64 bg-white dark:bg-oil text-baltic-sea dark:text-mercury-pinkish',
          'flex flex-col items-center justify-between h-full gap-4',
          'cursor-default select-none p-6 px-6',
        ])}
      >
        <div className="flex flex-col justify-center items-center gap-2">
          <MsIcon className="text-2xl font-bold" icon={'error'} />
          <h1 className="text-xl font-bold tracking-wider leading-loose">
            Problem detected
          </h1>
        </div>
        <p className="text-center">
          {error
            ? getErrorText(error?.key)
            : 'Something went wrong while loading the app. Please check the logs.'}
        </p>
        {error?.details && (
          <div className="w-full max-h-32 overflow-auto select-text text-balance break-words">
            <p className="text-teaberry text-center cursor-auto">
              {error.details}
            </p>
          </div>
        )}

        <Button
          color="malachite"
          onClick={() => {
            exit(0);
          }}
          className="mt-auto"
        >
          Close
        </Button>
      </div>
    </div>
  );
}

export default StartupError;
