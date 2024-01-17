import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { routes } from '../../constants';
import { useMainDispatch, useMainState } from '../../contexts';
import { StateDispatch } from '../../types';
import { Switch } from '../../ui';
import SettingsGroup from './SettingsGroup';

function Settings() {
  const state = useMainState();
  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');

  const [entrySelector, setEntrySelector] = useState(state.entrySelector);
  const [wgEnabled, setWgEnabled] = useState(state.wgEnabled);
  const [autoConnect, setAutoConnect] = useState(state.autoConnect);
  const [monitoring, setMonitoring] = useState(state.monitoring);

  useEffect(() => {
    setEntrySelector(state.entrySelector);
    setWgEnabled(state.wgEnabled);
    setAutoConnect(state.autoConnect);
    setMonitoring(state.monitoring);
  }, [state]);

  const handleEntrySelectorChange = async () => {
    const isSelected = !state.entrySelector;
    dispatch({ type: 'set-entry-selector', entrySelector: isSelected });
    invoke<void>('set_entry_location_selector', {
      entrySelector: isSelected,
    }).catch((e) => {
      console.log(e);
    });
  };

  const handleAutoConnectChanged = async () => {
    const isSelected = !state.autoConnect;
    dispatch({ type: 'set-auto-connect', autoConnect: isSelected });
    invoke<void>('set_auto_connect', { autoConnect: isSelected }).catch((e) => {
      console.log(e);
    });
  };

  const handleMonitoringChanged = async () => {
    const isSelected = !state.monitoring;
    dispatch({ type: 'set-monitoring', monitoring: isSelected });
    invoke<void>('set_monitoring', { monitoring: isSelected }).catch((e) => {
      console.log(e);
    });
  };

  const handleWgEnabledChange = async () => {
    dispatch({ type: 'set-wg-enabled', enabled: !state.wgEnabled });
    invoke<void>('enable_wireguard', {
      enabled: !state.wgEnabled,
    }).catch((e) => {
      console.log(e);
    });
  };

  return (
    <div className="h-full flex flex-col mt-2 gap-6">
      <SettingsGroup
        settings={[
          {
            title: t('auto-connect.title'),
            desc: t('auto-connect.desc'),
            leadingIcon: 'hdr_auto',
            disabled: true,
            trailing: (
              <Switch
                checked={autoConnect}
                onChange={handleAutoConnectChanged}
                disabled
              />
            ),
          },
          {
            title: t('entry-selector.title'),
            desc: t('entry-selector.desc'),
            leadingIcon: 'looks_two',
            trailing: (
              <Switch
                checked={entrySelector}
                onChange={handleEntrySelectorChange}
              />
            ),
          },
          {
            title: t('enable-wireguard.title'),
            desc: t('enable-wireguard.desc'),
            leadingIcon: 'vpn_key',
            trailing: (
              <Switch checked={wgEnabled} onChange={handleWgEnabledChange} />
            ),
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('display-theme'),
            leadingIcon: 'contrast',
            onClick: () => {
              navigate(routes.display);
            },
            trailing: (
              <div className="font-icon text-2xl cursor-pointer">
                arrow_right
              </div>
            ),
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('logs'),
            leadingIcon: 'sort',
            onClick: () => {
              navigate(routes.logs);
            },
            trailing: (
              <div className="font-icon text-2xl cursor-pointer">
                arrow_right
              </div>
            ),
            disabled: true,
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('feedback'),
            leadingIcon: 'question_answer',
            onClick: () => {
              navigate(routes.feedback);
            },
            trailing: (
              <div className="font-icon text-2xl cursor-pointer">
                arrow_right
              </div>
            ),
            disabled: true,
          },
          {
            title: t('error-reporting.title'),
            desc: t('error-reporting.desc'),
            leadingIcon: 'error',
            disabled: true,
            trailing: (
              <Switch
                checked={monitoring}
                onChange={handleMonitoringChanged}
                disabled
              />
            ),
          },
          {
            title: t('faq'),
            leadingIcon: 'help',
            disabled: true,
            trailing: (
              <div className="font-icon text-2xl cursor-pointer">launch</div>
            ),
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('legal'),
            onClick: () => {
              navigate(routes.legal);
            },
            disabled: true,
            trailing: (
              <div className="font-icon text-2xl cursor-pointer">
                arrow_right
              </div>
            ),
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('quit'),
            onClick: () => {
              //TODO shutdown gracefully
            },
            disabled: true,
          },
        ]}
      />
      <div className="flex grow flex-col justify-end text-comet text-sm tracking-tight leading-tight mb-4">
        Version {state.version}
      </div>
    </div>
  );
}

export default Settings;
