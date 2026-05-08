import { useCallback, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { AnimatePresence, motion } from 'motion/react';
import { type } from '@tauri-apps/plugin-os';
import { Command } from '@tauri-apps/plugin-shell';
import PageAnim from '../../../ui/PageAnim';
import SettingsMenuCard from '../../../ui/SettingsMenuCard';
import Switch from '../../../ui/Switch';
import { useDialog } from '../../../contexts';
import { Spinner } from '../../../ui';
import { useToast } from '../../../hooks/index';
import InfoDialog from './InfoDialog';
import LaunchConfirmDialog from './LaunchConfirmDialog';
import AppItem, { AppEntry } from './AppItem';
import { parseExecArgs, useSplitTunnel } from './utils';
import { PROBLEMATIC_APPS } from './utils/constants';

function SplitTunneling() {
  const os = type();

  const { t } = useTranslation('settings');
  const { isOpen, close } = useDialog();
  const { add: addToast } = useToast();

  const { apps, enabled, loading, setEnabled, add, remove, isSupported } =
    useSplitTunnel();

  const [runningApps, setRunningApps] = useState<Record<string, number[]>>({});
  const [pendingLaunchApp, setPendingLaunchApp] = useState<AppEntry | null>(
    null,
  );

  const spawnApp = useCallback(
    async (app: AppEntry) => {
      try {
        const command = Command.create(
          'nym-exclude',
          parseExecArgs(app.executable_path),
        );

        command.on('close', (data) => {
          console.info('[nym-exclude] process closed with code', data.code);
          setRunningApps((prev) => {
            const pids = prev[app.name];
            if (!pids) return prev;
            const updated = pids.filter((p) => p !== child.pid);
            if (updated.length === 0) {
              // eslint-disable-next-line @typescript-eslint/no-unused-vars
              const { [app.name]: _, ...rest } = prev;
              return rest;
            }
            return { ...prev, [app.name]: updated };
          });
        });

        command.on('error', (error) => {
          console.error('[nym-exclude] process error', error);
        });

        const child = await command.spawn();
        console.info('[nym-exclude] spawned PID', child.pid, 'for', app.name);

        setRunningApps((prev) => ({
          ...prev,
          [app.name]: [...(prev[app.name] || []), child.pid],
        }));
      } catch (error) {
        console.error('[nym-exclude] Failed to execute command', error);
        addToast({
          title: t('split-tunneling.error.failed-to-open-app'),
          type: 'error',
        });
      }
    },
    [addToast, t],
  );

  const handleLaunch = useCallback(
    async (app: AppEntry) => {
      const hasRunningPids = (runningApps[app.name]?.length ?? 0) > 0;
      const isProblematic = PROBLEMATIC_APPS.WITH_WARNING.has(
        app.executable_path.split('/').pop() || '',
      );

      if (isProblematic && !hasRunningPids) {
        setPendingLaunchApp(app);
      } else {
        await spawnApp(app);
      }
    },
    [runningApps, spawnApp],
  );

  const handleLaunchConfirm = useCallback(async () => {
    if (pendingLaunchApp) {
      await spawnApp(pendingLaunchApp);
    }
    setPendingLaunchApp(null);
  }, [pendingLaunchApp, spawnApp]);

  const handleLaunchCancel = useCallback(() => {
    setPendingLaunchApp(null);
  }, []);

  const sectionRefs = useRef<Record<string, HTMLDivElement | null>>({});

  const groupedApps = useMemo(() => {
    const groups: Record<string, AppEntry[]> = {};
    apps.forEach((app) => {
      const letter = app.name[0].toUpperCase();
      if (!groups[letter]) groups[letter] = [];
      groups[letter].push(app);
    });
    return groups;
  }, [apps]);

  const letters = useMemo(
    () => Object.keys(groupedApps).sort((a, b) => a.localeCompare(b)),
    [groupedApps],
  );

  const handleStateChange = async (app: AppEntry, state: AppEntry['state']) => {
    if (state === 'included') {
      await add(app);
    } else {
      await remove(app);
    }
  };

  const scrollToSection = (letter: string) => {
    sectionRefs.current[letter]?.scrollIntoView({ behavior: 'smooth' });
  };

  const handleEnableChange = async () => {
    await setEnabled(!enabled);
  };

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Spinner />
      </div>
    );
  }

  if (!isSupported) {
    return <p>Split tunneling is not supported on this platform</p>;
  }

  return (
    <>
      <InfoDialog
        isOpen={isOpen('split-tunneling-info')}
        onClose={() => close('split-tunneling-info')}
      />

      <LaunchConfirmDialog
        isOpen={pendingLaunchApp !== null}
        appName={pendingLaunchApp?.name ?? ''}
        onConfirm={handleLaunchConfirm}
        onCancel={handleLaunchCancel}
      />

      {/* Enable split tunneling on Windows only*/}
      {os === 'windows' && (
        <SettingsMenuCard
          title={t('split-tunneling.enable')}
          leadingIcon="call_split"
          trailingComponent={
            <Switch checked={enabled} onChange={handleEnableChange} />
          }
          onClick={handleEnableChange}
        />
      )}

      {/* Description */}
      <p className="text-text-secondary text-sm">
        {os === 'linux'
          ? t('split-tunneling.description-linux')
          : t('split-tunneling.description-windows')}
      </p>

      {/* Exclude warning */}
      <p className="text-cheddar dark:text-king-nacho bg-mercury/40 dark:bg-mine-shaft/60 rounded-lg p-3 text-sm">
        {t('split-tunneling.exclude-warning')}
      </p>

      {/* Apps section */}
      <AnimatePresence initial={false}>
        {(enabled || os === 'linux') && (
          <motion.div
            key="apps-section"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.15, ease: 'easeInOut' }}
            className="flex flex-col gap-2"
          >
            <p className="text-text-primary text-base font-semibold select-none">
              {t('split-tunneling.apps')} ({apps.length})
            </p>

            {/* App list with alphabetical sidebar */}
            <div className="flex items-stretch gap-0">
              {/* App list */}
              <div className="min-w-0 flex-1 overflow-hidden rounded-lg">
                {letters.map((letter) => (
                  <div
                    key={letter}
                    ref={(el) => {
                      sectionRefs.current[letter] = el;
                    }}
                  >
                    {/* Section divider */}
                    <div className="bg-mercury/40 dark:bg-mine-shaft/60 px-4 py-1">
                      <span className="text-text-secondary text-xs select-none">
                        {letter}
                      </span>
                    </div>

                    {/* Apps in this section */}
                    {groupedApps[letter].map((app, i) => (
                      <div key={app.name}>
                        <AppItem
                          app={app}
                          onStateChange={handleStateChange}
                          isRunning={(runningApps[app.name]?.length ?? 0) > 0}
                          onLaunch={handleLaunch}
                        />
                        {i < groupedApps[letter].length - 1 && (
                          <div className="bg-mercury/60 mx-4 h-px dark:bg-white/5" />
                        )}
                      </div>
                    ))}
                  </div>
                ))}
              </div>

              {/* Alphabetical sidebar */}
              <div className="sticky top-0 ml-3.5 flex w-5 flex-col items-center justify-between gap-1.5 self-start">
                {letters.map((letter) => (
                  <button
                    key={letter}
                    className={clsx(
                      'h-4 w-full cursor-default text-center text-xs select-none',
                      'text-text-secondary hover:text-baltic-sea dark:hover:text-white',
                      'transition-noborder',
                    )}
                    onClick={() => scrollToSection(letter)}
                  >
                    {letter}
                  </button>
                ))}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}

function SplitTunnelingAnimWrapper() {
  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-4">
      <SplitTunneling />
    </PageAnim>
  );
}

export default SplitTunnelingAnimWrapper;
