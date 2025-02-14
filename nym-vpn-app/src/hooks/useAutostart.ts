import { useEffect, useState } from 'react';
import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart';

/* thin wrapper around tauri autostart plugin */
function useAutostart() {
  const [enabled, setEnabled] = useState(false);

  useEffect(() => {
    const init = async () => {
      const enabled = await isEnabled();
      setEnabled(enabled);
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
    setEnabled(!enabled);
  };

  return { enabled, toggle };
}

export default useAutostart;
