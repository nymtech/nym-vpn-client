import { invoke } from '@tauri-apps/api/core';
import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { dispatch, useAppStore } from '../../../../store';
import { App } from '../../../../types/tauri';
import { BackendError } from '../../../../types/util';
import { AppEntry } from '../AppItem';
import { useI18nError, useToast } from '../../../../hooks/index';

export const useSplitTunnel = () => {
  const { t } = useTranslation('settings');
  const splitTunnel = useAppStore((s) => s.splitTunnel);
  const { enabled, apps: splitTunnelApps } = splitTunnel;
  const { add: addToast } = useToast();
  const { tE } = useI18nError();

  const [installedApps, setInstalledApps] = useState<App[]>([]);
  const [loading, setLoading] = useState(false);
  const [isSupported, setIsSupported] = useState(false);

  useEffect(() => {
    (async () => {
      setLoading(true);

      try {
        const isSupported = await invoke<boolean>('is_split_tunnel_supported');
        setIsSupported(isSupported);

        if (!isSupported) {
          setLoading(false);
          return;
        }

        const appList = await invoke<App[]>('get_app_list');
        setInstalledApps(appList);
      } catch (err: unknown) {
        console.error('Failed to get app list', err);
        addToast({
          title: t('split-tunnel.failed-to-get-app-list', { ns: 'errors' }),
          type: 'error',
        });
      } finally {
        setLoading(false);
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const appList: AppEntry[] = useMemo(() => {
    return installedApps.map<AppEntry>((app) => ({
      ...app,
      state: splitTunnelApps.some(
        (existing) => existing.path === app.executable_path,
      )
        ? 'included'
        : 'excluded',
    }));
  }, [splitTunnelApps, installedApps]);

  const setEnabled = async (enabled: boolean) => {
    try {
      await invoke('set_enable_split_tunnel', { enabled });
      dispatch({ type: 'set-enable-split-tunnel', enabled });
    } catch (error) {
      console.error('Failed to set split tunneling enabled', error);
      addToast({
        title: t('split-tunneling.error.failed-to-enable-split-tunneling'),
        type: 'error',
      });
    }
  };

  const add = async (app: AppEntry) => {
    if (app.state === 'included') return;
    try {
      await invoke('add_app_to_split_tunnel', {
        app: { path: app.executable_path },
      });
      dispatch({
        type: 'set-split-tunnel-apps',
        apps: [...splitTunnelApps, { path: app.executable_path }],
      });
    } catch (error) {
      console.error('Failed to add app to split tunneling', error);
      addToast({
        title: t('split-tunneling.error.failed-to-add-app-to-split-tunneling'),
        type: 'error',
      });
    }
  };

  // Opens a native file dialog (backend side) to pick a custom executable and
  // add it to the app-side split-tunnel list. Returns silently if the user
  // cancels the dialog.
  const addCustomApp = async () => {
    try {
      const app = await invoke<App | null>('add_custom_split_tunnel_app');
      if (!app) return; // user cancelled the dialog

      // Refetch so the new app is merged + sorted consistently with the backend.
      const appList = await invoke<App[]>('get_app_list');
      setInstalledApps(appList);

      addToast({
        title: t('split-tunneling.custom-app-added', { name: app.name }),
        type: 'info',
      });
    } catch (err: unknown) {
      console.error('Failed to add custom split tunnel app', err);
      const error = err as BackendError;
      addToast({
        title: error?.key
          ? tE(error.key)
          : t('split-tunneling.error.failed-to-add-custom-app'),
        type: 'error',
      });
    }
  };

  // Removes a user-added (custom) app from the app-side split-tunnel list.
  const removeCustomApp = async (app: AppEntry) => {
    try {
      await invoke('remove_custom_split_tunnel_app', {
        path: app.executable_path,
      });

      // Refetch so the list reflects the removal (discovered apps untouched).
      const appList = await invoke<App[]>('get_app_list');
      setInstalledApps(appList);
    } catch (err: unknown) {
      console.error('Failed to remove custom split tunnel app', err);
      const error = err as BackendError;
      addToast({
        title: error?.key
          ? tE(error.key)
          : t('split-tunneling.error.failed-to-remove-custom-app'),
        type: 'error',
      });
    }
  };

  const remove = async (app: AppEntry) => {
    if (app.state === 'excluded') return;
    try {
      await invoke('remove_app_from_split_tunnel', {
        app: { path: app.executable_path },
      });
      dispatch({
        type: 'set-split-tunnel-apps',
        apps: splitTunnelApps.filter(
          (existing) => existing.path !== app.executable_path,
        ),
      });
    } catch (error) {
      console.error('Failed to remove app from split tunneling', error);
      addToast({
        title: t(
          'split-tunneling.error.failed-to-remove-app-from-split-tunneling',
        ),
        type: 'error',
      });
    }
  };

  return {
    apps: appList,
    enabled,
    setEnabled,
    add,
    addCustomApp,
    remove,
    removeCustomApp,
    loading,
    isSupported,
  };
};
