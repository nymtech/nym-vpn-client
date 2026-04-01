import { invoke } from '@tauri-apps/api/core';
import { useEffect, useMemo, useState } from 'react';
import {
  useInAppNotify,
  useMainDispatch,
  useMainState,
} from '../../../../contexts';
import { StateDispatch } from '../../../../types';
import { App } from '../../../../types/tauri';
import { AppEntry } from '../AppItem';

export const useSplitTunnel = () => {
  const {
    splitTunnel: { enabled, apps: splitTunnelApps },
  } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { push } = useInAppNotify();

  const [installedApps, setInstalledApps] = useState<App[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    (async () => {
      setLoading(true);

      try {
        const appList = await invoke<App[]>('get_app_list');
        setInstalledApps(appList);
      } catch (err: unknown) {
        console.error('Failed to get app list', err);
        push({
          message: 'Failed to get app list',
          close: true,
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
      name: app.name,
      executable_path: app.executable_path,
      icon: app.icon,
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
      push({
        message: 'Failed to set split tunneling enabled',
        close: true,
        type: 'error',
      });
    }
  };

  const add = async (app: AppEntry) => {
    if (app.state === 'included') return;
    try {
      await invoke('add_app_to_split_tunnel', { app: { path: app.executable_path } });
      dispatch({
        type: 'set-split-tunnel-apps',
        apps: [...splitTunnelApps, { path: app.executable_path }],
      });
    } catch (error) {
      console.error('Failed to add app to split tunneling', error);
      push({
        message: 'Failed to add app to split tunneling',
        close: true,
        type: 'error',
      });
    }
  }

  const remove = async (app: AppEntry) => {
    if (app.state === 'excluded') return;
    try {
      await invoke('remove_app_from_split_tunnel', { app: { path: app.executable_path } });
      dispatch({
        type: 'set-split-tunnel-apps',
        apps: splitTunnelApps.filter((existing) => existing.path !== app.executable_path),
      });
    } catch (error) {
      console.error('Failed to remove app from split tunneling', error);
      push({
        message: 'Failed to remove app from split tunneling',
        close: true,
        type: 'error',
      });
    }
  };

  return {
    apps: appList,
    enabled,
    setEnabled,
    add,
    remove,
    loading,
  };
};
