import { useEffect } from 'react';
import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart';
import { dispatch, useMainState } from '../store';

/* thin wrapper around tauri autostart plugin */
function useAutostart() {
  const { autostart } = useMainState();

  useEffect(() => {
    const init = async () => {
      const enabled = await isEnabled();
      dispatch({ type: 'set-autostart', enabled });
    };
    init();
  }, []);

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
