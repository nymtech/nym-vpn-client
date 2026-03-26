import { invoke } from '@tauri-apps/api/core';
import { useMainDispatch, useMainState } from '../../../../contexts';
import { StateDispatch } from '../../../../types';
import { SplitApp } from '../../../../types/tauri';

export const useSplitTunnel = () => {
  const {
    splitTunnel: { enabled, apps },
  } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  const setEnabled = (enabled: boolean) => {
    dispatch({ type: 'set-enable-split-tunnel', enabled });
    invoke('set_enable_split_tunnel', { enabled });
  };

  const add = (app: SplitApp) => {
    if (!apps.some((existing) => existing.path === app.path)) {
      dispatch({ type: 'set-split-tunnel-apps', apps: [...apps, app] });
      invoke('add_app_to_split_tunnel', { app });
    }
  };

  const remove = (app: SplitApp) => {
    dispatch({
      type: 'set-split-tunnel-apps',
      apps: apps.filter((existing) => existing.path !== app.path),
    });
    invoke('remove_app_from_split_tunnel', { app });
  };

  return {
    apps,
    enabled,
    setEnabled,
    add,
    remove,
  };
};
