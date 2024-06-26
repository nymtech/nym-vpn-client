import { useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api';
import { useTranslation } from 'react-i18next';
import {
  useMainDispatch,
  useMainState,
  useNotifications,
} from '../../contexts';
import { StateDispatch, VpnMode } from '../../types';
import { MixnetIcon } from '../../assets';
import { RadioGroup, RadioGroupOption } from '../../ui';
import { useThrottle } from '../../hooks';
import { HomeThrottleDelay } from '../../constants';

function NetworkModeSelect() {
  const state = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const [loading, setLoading] = useState(false);
  const { push } = useNotifications();

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

  const showSnackbar = useThrottle(
    () => {
      let text = '';
      switch (state.state) {
        case 'Connected':
          text = t('snackbar-disabled-message.connected');
          break;
        case 'Connecting':
          text = t('snackbar-disabled-message.connecting');
          break;
        case 'Disconnecting':
          text = t('snackbar-disabled-message.disconnecting');
          break;
      }
      push({
        text,
        position: 'top',
      });
    },
    HomeThrottleDelay,
    [state.state],
  );

  const handleDisabledState = () => {
    if (state.state !== 'Disconnected') {
      showSnackbar();
    }
  };

  const vpnModes = useMemo<RadioGroupOption<VpnMode>[]>(() => {
    return [
      {
        key: 'Mixnet',
        label: t('mixnet-mode.title'),
        desc: t('mixnet-mode.desc'),
        disabled: state.state !== 'Disconnected' || loading,
        icon: (
          <MixnetIcon className="w-7 h-7 fill-baltic-sea dark:fill-mercury-pinkish" />
        ),
      },
      {
        key: 'TwoHop',
        label: t('twohop-mode.title'),
        desc: t('twohop-mode.desc'),
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
    // eslint-disable-next-line jsx-a11y/click-events-have-key-events,jsx-a11y/no-static-element-interactions
    <div className="select-none" onClick={handleDisabledState}>
      <RadioGroup
        defaultValue={state.vpnMode}
        options={vpnModes}
        onChange={(mode) => {
          handleNetworkModeChange(mode);
        }}
        rootLabel={t('select-network-label')}
      />
    </div>
  );
}

export default NetworkModeSelect;
