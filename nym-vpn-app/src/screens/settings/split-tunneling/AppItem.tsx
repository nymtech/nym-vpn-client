import clsx from 'clsx';
import { convertFileSrc } from '@tauri-apps/api/core';
import { type } from '@tauri-apps/plugin-os';
import { useTranslation } from 'react-i18next';
import { useMemo } from 'react';
import MsIcon from '../../../ui/MsIcon';
import { App } from '../../../types';
import { PROBLEMATIC_APPS } from './utils/constants';

export type AppEntry = App & {
  state: 'excluded' | 'included';
};

type AppItemProps = {
  app: AppEntry;
  onStateChange: (
    app: AppEntry,
    state: 'excluded' | 'included',
  ) => Promise<void>;
  isRunning?: boolean;
  onLaunch?: (app: AppEntry) => Promise<void>;
};

function AppItem({ app, onStateChange, isRunning, onLaunch }: AppItemProps) {
  const { t } = useTranslation('settings');
  const os = type();

  const isProblematic = useMemo(
    () =>
      PROBLEMATIC_APPS.DISABLED.has(app.executable_path.split('/').pop() || ''),
    [app.executable_path],
  );

  const handleClick = async () => {
    if (os === 'linux' && onLaunch) await onLaunch(app);
  };

  return (
    <div
      className={clsx(
        'dark:bg-charcoal flex items-center gap-3 bg-white px-4 py-3',
        os === 'linux' && 'dark:hover:bg-charcoal/75 hover:bg-black/10',
        isProblematic && 'cursor-not-allowed opacity-50',
      )}
      onClick={handleClick}
    >
      <div className="relative flex h-7 w-7 items-center justify-center">
        {app.icon && (
          <img
            src={convertFileSrc(app.icon)}
            alt={app.name}
            className="h-full w-full"
          />
        )}
        {!app.icon && (
          <div className="bg-faded-lavender dark:bg-ash text-text-primary flex h-full w-full items-center justify-center rounded-md text-sm leading-none">
            {app.name[0].toUpperCase()}
          </div>
        )}
        {os === 'linux' && (
          <div
            className={clsx(
              'bg-malachite-moss absolute right-0 bottom-0 h-2 w-2 rounded-full',
              {
                'bg-malachite-moss animate-pulse duration-1000': isRunning,
                'bg-ash dark:bg-mercury': !isRunning,
              },
            )}
          ></div>
        )}
      </div>
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <span className="text-text-primary flex-1 truncate text-sm select-none">
          {app.name}
        </span>
        {isProblematic && (
          <span className="text-cheddar dark:text-king-nacho text-xs">
            {t('split-tunneling.problematic-app')}
          </span>
        )}
      </div>
      {os === 'linux' && (
        <MsIcon icon="open_in_new" className="text-bombay shrink-0 text-base" />
      )}
      {/* Only Windows can include/exclude apps from inside the app */}
      {/* Linux uses custom app launcher to launch the app and immediately exclude it from the tunnel */}
      {os === 'windows' && (
        <div className="border-iron dark:border-bombay flex items-center rounded-lg border">
          <button
            className={clsx(
              'transition-noborder border-r-iron dark:border-r-bombay flex h-full w-full cursor-default items-center justify-center rounded-l-lg border-r px-2',
              app.state === 'included'
                ? 'bg-aphrodisiac/20 text-aphrodisiac'
                : 'text-text-secondary',
            )}
            onClick={() => onStateChange(app, 'included')}
            aria-label={`Exclude ${app.name} from VPN`}
          >
            <MsIcon icon="block" className="text-base" />
          </button>
          <button
            className={clsx(
              'transition-noborder flex h-full w-full cursor-default items-center justify-center rounded-r-lg px-2',
              app.state === 'excluded'
                ? 'bg-malachite-moss/15 dark:bg-malachite/15 text-primary'
                : 'text-text-secondary',
            )}
            onClick={() => onStateChange(app, 'excluded')}
            aria-label={`Include ${app.name} in VPN`}
          >
            <MsIcon icon="shield" className="text-base" />
          </button>
        </div>
      )}
    </div>
  );
}

export default AppItem;
