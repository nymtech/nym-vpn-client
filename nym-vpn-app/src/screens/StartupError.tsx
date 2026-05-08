import { useEffect } from 'react';
import clsx from 'clsx';
import { exit } from '@tauri-apps/plugin-process';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Button, MsIcon } from '../ui';

function getErrorText(key: StartupError['key']) {
  switch (key) {
    case 'db-open':
      return 'Failed to open the application database.';
    case 'db-locked':
      return 'The application is likely already running. Check your system tray or task manager.';
    default:
      return 'Internal error. Please contact support and share the app logs.';
  }
}

let initialized = false;

function StartupError({
  error,
  theme,
}: {
  error: StartupError;
  theme: 'light' | 'dark' | null;
}) {
  useEffect(() => {
    if (initialized) {
      return;
    }
    initialized = true;
    const window = getCurrentWindow();
    console.info(`show window ${window.label}`);
    window.show().catch((e: unknown) => {
      console.error(`failed to show error window: ${e}`);
    });
  }, []);

  return (
    <div
      className={clsx([theme === 'dark' && 'dark', 'h-full'])}
      data-testid="startup-error-container"
      data-test-theme={theme}
    >
      <div
        className={clsx([
          'dark:bg-charcoal text-text-primary min-w-64 bg-white',
          'flex h-full flex-col items-center justify-between gap-4',
          'cursor-default p-6 px-6 select-none',
        ])}
        data-testid="startup-error-content"
      >
        <div
          className="flex flex-col items-center justify-center gap-2"
          data-testid="startup-error-header"
        >
          <MsIcon
            className="text-2xl font-medium"
            icon={'error'}
            data-testid="startup-error-icon"
          />
          <h1
            className="text-xl leading-loose font-medium tracking-wider"
            data-testid="startup-error-title"
          >
            Problem detected
          </h1>
        </div>
        <p className="text-center" data-testid="startup-error-message">
          {error
            ? getErrorText(error?.key)
            : 'Something went wrong while loading the app. Please contact support and share the app logs.'}
        </p>
        {error?.detail && (
          <div
            className="max-h-44 w-full overflow-auto text-balance break-words select-text"
            data-testid="startup-error-details"
          >
            <p className="text-aphrodisiac cursor-auto text-center">
              {error.detail}
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
