import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api';
import { exit } from '@tauri-apps/api/process';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { routes } from '../../constants';
import { useMainDispatch, useMainState } from '../../contexts';
import { CmdError, StateDispatch } from '../../types';
import { MsIcon, Switch } from '../../ui';
import SettingsGroup from './SettingsGroup';

function Settings() {
  const state = useMainState();
  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');

  const [entrySelector, setEntrySelector] = useState(state.entrySelector);
  const [autoConnect, setAutoConnect] = useState(state.autoConnect);

  useEffect(() => {
    setEntrySelector(state.entrySelector);
    setAutoConnect(state.autoConnect);
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

  const handleQuit = async () => {
    if (state.state === 'Connected') {
      // TODO add a timeout to prevent the app from hanging
      // in bad disconnect scenarios
      dispatch({ type: 'disconnect' });
      invoke('disconnect')
        .then(async (result) => {
          console.log('disconnect result');
          console.log(result);
          await exit(0);
        })
        .catch(async (e: CmdError) => {
          console.warn(`backend error: ${e.source} - ${e.message}`);
          await exit(1);
        });
    } else {
      await exit(0);
    }
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
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('display-theme'),
            leadingIcon: 'contrast',
            onClick: async () => {
              navigate(routes.display);
            },
            trailing: <MsIcon icon="arrow_right" />,
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('logs'),
            leadingIcon: 'sort',
            onClick: async () => {
              navigate(routes.logs);
            },
            trailing: <MsIcon icon="arrow_right" />,
            disabled: true,
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('feedback.title'),
            leadingIcon: 'edit_note',
            onClick: async () => {
              navigate(routes.feedback);
            },
            trailing: <MsIcon icon="arrow_right" />,
          },
          {
            title: t('support.title'),
            leadingIcon: 'question_answer',
            onClick: async () => {
              navigate(routes.support);
            },
            trailing: <MsIcon icon="arrow_right" />,
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('legal'),
            onClick: async () => {
              navigate(routes.legal);
            },
            trailing: <MsIcon icon="arrow_right" />,
            disabled: true,
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('quit'),
            onClick: handleQuit,
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
