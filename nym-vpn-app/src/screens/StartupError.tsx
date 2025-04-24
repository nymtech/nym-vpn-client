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
    <div
      className={clsx([theme === 'dark' && 'dark', 'h-full'])}
      data-testid="startup-error-container"
      data-theme={theme}
    >
      <div
        className={clsx([
          'min-w-64 bg-white dark:bg-charcoal text-baltic-sea dark:text-white',
          'flex flex-col items-center justify-between h-full gap-4',
          'cursor-default select-none p-6 px-6',
        ])}
        data-testid="startup-error-content"
      >
        <div
          className="flex flex-col justify-center items-center gap-2"
          data-testid="startup-error-header"
        >
          <MsIcon
            className="text-2xl font-medium"
            icon={'error'}
            data-testid="startup-error-icon"
          />
          <h1
            className="text-xl font-medium tracking-wider leading-loose"
            data-testid="startup-error-title"
          >
            Problem detected
          </h1>
        </div>
        <p className="text-center" data-testid="startup-error-message">
          {error
            ? getErrorText(error?.key)
            : 'Something went wrong while loading the app. Please check the logs.'}
        </p>
        {error?.details && (
          <div
            className="w-full max-h-32 overflow-auto select-text text-balance break-words"
            data-testid="startup-error-details"
          >
            <p className="text-aphrodisiac text-center cursor-auto">
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
          data-testid="startup-error-close-button"
        >
          Close
        </Button>
      </div>
    </div>
  );
}

export default StartupError;
