import { useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api';
import { useTranslation } from 'react-i18next';
import { useMainDispatch, useMainState } from '../../contexts';
import { StateDispatch, VpnMode } from '../../types';
import { MixnetIcon } from '../../assets';
import { RadioGroup, RadioGroupOption, RadioGroupOptionCursor } from '../../ui';

function NetworkModeSelect() {
  const state = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const [loading, setLoading] = useState(false);

  const { t } = useTranslation('home');

  const handleNetworkModeChange = async (value: VpnMode) => {
    if (state.state === 'Disconnected' && value !== state.vpnMode) {
      setLoading(true);
      try {
        await invoke<void>('set_vpn_mode', { mode: value });
        dispatch({ type: 'set-vpn-mode', mode: value });
      } catch (e) {
        console.warn(e);
      } finally {
        setLoading(false);
      }
    }
  };

  const vpnModes = useMemo<RadioGroupOption<VpnMode>[]>(() => {
    const getCursorMode = (): RadioGroupOptionCursor => {
      return state.state === 'Disconnected' ? 'pointer' : 'default';
    };

    return [
      {
        key: 'Mixnet',
        label: t('mixnet-mode.title'),
        desc: t('mixnet-mode.desc'),
        cursor: getCursorMode(),
        disabled: state.state !== 'Disconnected' || loading,
        icon: (
          <MixnetIcon className="w-7 h-7 fill-baltic-sea dark:fill-mercury-pinkish" />
        ),
      },
      {
        key: 'TwoHop',
        label: t('twohop-mode.title'),
        desc: t('twohop-mode.desc'),
        cursor: getCursorMode(),
        disabled: state.state !== 'Disconnected' || loading,
        icon: (
          <span className="font-icon text-3xl text-baltic-sea dark:text-mercury-pinkish">
            security
          </span>
        ),
      },
    ];
  }, [loading, state.state, t]);

  return (
    <div className="select-none">
      <RadioGroup
        defaultValue={state.vpnMode}
        options={vpnModes}
        onChange={handleNetworkModeChange}
        rootLabel={t('select-network-label')}
      />
    </div>
  );
}

export default NetworkModeSelect;
