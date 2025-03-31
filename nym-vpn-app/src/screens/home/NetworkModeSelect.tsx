import { useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { Button } from '@headlessui/react';
import { useGateways, useMainDispatch, useMainState } from '../../contexts';
import { StateDispatch, VpnMode } from '../../types';
import { RadioGroup, RadioGroupOption } from '../../ui';
import MsIcon from '../../ui/MsIcon';
import { S_STATE } from '../../static';
import ModeDetailsDialog from './ModeDetailsDialog';
import { useActionToast } from './util';

function NetworkModeSelect() {
  const { state, vpnMode, daemonStatus } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const { fetch } = useGateways();

  const [isDialogModesOpen, setIsDialogModesOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const toast = useActionToast('mode-select');

  const { t } = useTranslation('home');

  const handleNetworkModeChange = async (value: VpnMode) => {
    if (state === 'Disconnected' && value !== vpnMode) {
      setLoading(true);
      try {
        await invoke<void>('set_vpn_mode', { mode: value });
        dispatch({ type: 'set-vpn-mode', mode: value });
        console.info(`vpn mode set to [${value}]`);
        if (value === 'mixnet') {
          fetch('mx-entry');
          fetch('mx-exit');
        } else {
          fetch('wg');
        }
      } catch (e) {
        console.warn(e);
      } finally {
        setLoading(false);
      }
    }
  };

  const handleDisabledState = () => {
    if (state !== 'Disconnected') {
      toast();
    }
  };

  const vpnModes = useMemo<RadioGroupOption<VpnMode>[]>(() => {
    const iconStyle = (checked: boolean) =>
      clsx(
        'font-icon text-3xl',
        checked
          ? 'text-malachite-moss dark:text-malachite'
          : 'text-baltic-sea dark:text-white',
      );

    return [
      {
        key: 'wg',
        label: t('fast-mode.title'),
        desc: t('fast-mode.desc'),
        disabled: state !== 'Disconnected' || loading,
        icon: (checked) => <span className={iconStyle(checked)}>speed</span>,
      },
      {
        key: 'mixnet',
        label: t('privacy-mode.title'),
        desc: t('privacy-mode.desc'),
        disabled: state !== 'Disconnected' || loading,
        icon: (checked) => (
          <span className={iconStyle(checked)}>visibility_off</span>
        ),
      },
    ];
  }, [loading, state, t]);

  return (
    <div>
      <div
        className={clsx([
          'flex flex-row items-center justify-between',
          'font-medium text-base text-baltic-sea dark:text-white mb-5 cursor-default',
        ])}
      >
        <label>{t('select-mode-label')}</label>
        <Button
          className="w-6 focus:outline-hidden cursor-default"
          onClick={() => setIsDialogModesOpen(true)}
        >
          <MsIcon
            icon="info"
            className={clsx([
              'text-xl',
              'text-iron dark:text-bombay transition duration-150',
              'opacity-90 dark:opacity-100 hover:opacity-100 hover:text-baltic-sea dark:hover:text-white',
            ])}
          />
        </Button>
      </div>
      <ModeDetailsDialog
        isOpen={isDialogModesOpen}
        onClose={() => setIsDialogModesOpen(false)}
      />
      <div className="select-none" onClick={handleDisabledState}>
        <RadioGroup
          key={`_${S_STATE.vpnModeInit}`}
          defaultValue={vpnMode}
          options={vpnModes}
          onChange={handleNetworkModeChange}
          radioIcons={false}
          disabled={daemonStatus === 'down'}
        />
      </div>
    </div>
  );
}

export default NetworkModeSelect;
