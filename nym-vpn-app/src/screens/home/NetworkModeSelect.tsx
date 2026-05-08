import { useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { useShallow } from 'zustand/react/shallow';
import { dispatch, useAppStore, useFetchGateways } from '../../store';
import { VpnMode } from '../../types';
import { ButtonIcon, RadioGroup, RadioGroupOption } from '../../ui';
import ModeDetailsDialog from './ModeDetailsDialog';

function NetworkModeSelect() {
  const { vpnMode, daemonStatus } = useAppStore(
    useShallow((s) => ({
      vpnMode: s.vpnMode,
      daemonStatus: s.daemonStatus,
    })),
  );
  const fetchGateways = useFetchGateways();

  const [isDialogModesOpen, setIsDialogModesOpen] = useState(false);
  const [loading, setLoading] = useState(false);

  const { t } = useTranslation('home');

  const handleNetworkModeChange = async (value: VpnMode) => {
    if (value !== vpnMode) {
      setLoading(true);
      try {
        await invoke<void>('set_vpn_mode', { mode: value });
        dispatch({ type: 'set-vpn-mode', mode: value });
        console.info(`vpn mode set to [${value}]`);
        if (value === 'mixnet') {
          fetchGateways('mx-entry');
          fetchGateways('mx-exit');
        } else {
          fetchGateways('wg');
        }
      } finally {
        setLoading(false);
      }
    }
  };

  const vpnModes = useMemo<RadioGroupOption<VpnMode>[]>(() => {
    const iconStyle = (checked: boolean) =>
      clsx(
        'font-icon text-2xl leading-none',
        checked ? 'text-primary' : 'text-bombay dark:text-iron',
      );

    return [
      {
        key: 'wg',
        label: t('fast-mode.title'),
        desc: t('fast-mode.desc'),
        disabled: loading,
        icon: (checked) => (
          <span
            className={iconStyle(checked)}
            data-testid="network-mode-fast-icon"
          >
            speed
          </span>
        ),
        descWrap: true,
      },
      {
        key: 'mixnet',
        label: t('privacy-mode.title'),
        desc: t('privacy-mode.desc'),
        disabled: loading,
        icon: (checked) => (
          <span
            className={iconStyle(checked)}
            data-testid="network-mode-privacy-icon"
          >
            visibility_off
          </span>
        ),
        descWrap: true,
      },
    ];
  }, [loading, t]);

  return (
    <div data-testid="network-mode-select-container">
      <div
        className={clsx([
          'flex flex-row items-center justify-between',
          'text-text-primary mb-5 cursor-default text-base font-medium',
        ])}
        data-testid="network-mode-label-container"
      >
        <label data-testid="network-mode-label">{t('select-mode-label')}</label>
        <ButtonIcon
          noDefaultSize
          icon="info"
          onClick={() => setIsDialogModesOpen(true)}
          color="chalk"
        />
      </div>
      <ModeDetailsDialog
        isOpen={isDialogModesOpen}
        onClose={() => setIsDialogModesOpen(false)}
      />
      <div
        className="select-none"
        data-testid="network-mode-radio-group-container"
      >
        <RadioGroup
          key={`_${vpnMode}`}
          defaultValue={vpnMode}
          options={vpnModes}
          onChange={handleNetworkModeChange}
          radioIcons={false}
          disabled={daemonStatus === 'down'}
          data-testid="network-mode-radio-group"
        />
      </div>
    </div>
  );
}

export default NetworkModeSelect;
