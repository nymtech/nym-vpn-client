import { invoke } from '@tauri-apps/api/core';
import { Switch } from '../../../ui';
import { useMainDispatch, useMainState } from '../../../contexts';
import { StateDispatch } from '../../../types';

function LewesProtocolSwitch() {
  const { enableLewesProtocol } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  const handleChange = (enabled: boolean) => {
    dispatch({ type: 'set-enable-lewes-protocol', enabled });
    invoke('set_enable_lewes_protocol', { enabled });
  };

  return (
    <div>
      <h3 className="text-lg text-baltic-sea dark:text-white font-medium">
        Lewes Protocol
      </h3>
      <p>{enableLewesProtocol ? 'Enabled' : 'Disabled'}</p>
      <Switch checked={enableLewesProtocol} onChange={handleChange} />
    </div>
  );
}

export default LewesProtocolSwitch;
