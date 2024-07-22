import { useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { Button } from '@headlessui/react';
import {
  useMainDispatch,
  useMainState,
  useNotifications,
} from '../../contexts';
import { StateDispatch, VpnMode } from '../../types';
import { RadioGroup, RadioGroupOption } from '../../ui';
import { useThrottle } from '../../hooks';
import { HomeThrottleDelay } from '../../constants';
import MsIcon from '../../ui/MsIcon';
import ModeDetailsDialog from './ModeDetailsDialog';

function NetworkModeSelect() {
  const state = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const [isDialogModesOpen, setIsDialogModesOpen] = useState(false);
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
        label: t('privacy-mode.title'),
        desc: t('privacy-mode.desc'),
        disabled: state.state !== 'Disconnected' || loading,
        icon: (
          <span className="font-icon text-3xl text-baltic-sea dark:text-mercury-pinkish">
            visibility_off
          </span>
        ),
      },
      {
        key: 'TwoHop',
        label: t('fast-mode.title'),
        desc: t('fast-mode.desc'),
        disabled: state.state !== 'Disconnected' || loading,
        icon: (
          <span className="font-icon text-3xl text-baltic-sea dark:text-mercury-pinkish">
            speed
          </span>
        ),
      },
    ];
  }, [loading, state.state, t]);

  return (
    <div>
      <div
        className={clsx([
          'flex flex-row items-center justify-between',
          'font-semibold text-base text-baltic-sea dark:text-white mb-5 cursor-default',
        ])}
      >
        <label>{t('select-mode-label')}</label>
        <Button
          className="w-6 focus:outline-none cursor-default"
          onClick={() => setIsDialogModesOpen(true)}
        >
          <MsIcon
            icon="info"
            className={clsx([
              'text-xl',
              'text-cement-feet dark:text-mercury-mist transition duration-150',
              'opacity-90 dark:opacity-100 hover:opacity-100 hover:text-gun-powder hover:dark:text-mercury-pinkish',
            ])}
          />
        </Button>
      </div>
      <ModeDetailsDialog
        isOpen={isDialogModesOpen}
        onClose={() => setIsDialogModesOpen(false)}
      />
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events,jsx-a11y/no-static-element-interactions */}
      <div className="select-none" onClick={handleDisabledState}>
        <RadioGroup
          defaultValue={state.vpnMode}
          options={vpnModes}
          onChange={handleNetworkModeChange}
          radioIcons={false}
        />
      </div>
    </div>
  );
}

export default NetworkModeSelect;
