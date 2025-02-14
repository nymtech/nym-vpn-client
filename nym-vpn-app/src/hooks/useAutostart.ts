import { useEffect } from 'react';
import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart';
import { useMainDispatch, useMainState } from '../contexts';
import { StateDispatch } from '../types';

/* thin wrapper around tauri autostart plugin */
function useAutostart() {
  const { autostart } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  useEffect(() => {
    const init = async () => {
      const enabled = await isEnabled();
      dispatch({ type: 'set-autostart', enabled });
    };
    init();
  }, [dispatch]);

  const toggle = async () => {
    const enabled = await isEnabled();
    if (enabled) {
      await disable();
    } else {
      await enable();
    }
    dispatch({ type: 'set-autostart', enabled: !enabled });
  };

  return { enabled: autostart, toggle };
}

export default useAutostart;
