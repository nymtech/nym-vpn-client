import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { kvSet } from '../../kvStore';
import { routes } from '../../router';
import { useMainDispatch, useMainState } from '../../contexts';
import { useExit } from '../../state';
import { StateDispatch } from '../../types';
import { Button, MsIcon, SettingsMenuCard, Switch } from '../../ui';
import SettingsGroup from './SettingsGroup';

function Settings() {
  const state = useMainState();
  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');

  const [entrySelector, setEntrySelector] = useState(state.entrySelector);
  const [autoConnect, setAutoConnect] = useState(state.autoConnect);
  const { exit } = useExit();

  useEffect(() => {
    setEntrySelector(state.entrySelector);
    setAutoConnect(state.autoConnect);
  }, [state]);

  const handleEntrySelectorChange = async () => {
    const isSelected = !state.entrySelector;
    dispatch({ type: 'set-entry-selector', entrySelector: isSelected });
    kvSet('EntryLocationEnabled', isSelected).catch((e) => {
      console.warn(e);
    });
  };

  const handleAutoConnectChanged = async () => {
    const isSelected = !state.autoConnect;
    dispatch({ type: 'set-auto-connect', autoConnect: isSelected });
    kvSet('Autoconnect', isSelected).catch((e) => {
      console.warn(e);
    });
  };

  return (
    <div className="h-full flex flex-col mt-2 gap-6">
      {import.meta.env.APP_LOGIN_ENABLED === 'true' && (
        <Button onClick={async () => navigate('/login')}>
          {t('login-button')}
        </Button>
      )}
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
      <SettingsMenuCard
        title={t('display-theme')}
        onClick={async () => {
          navigate(routes.display);
        }}
        leadingIcon="contrast"
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard
        title={t('logs')}
        onClick={async () => {
          navigate(routes.logs);
        }}
        leadingIcon="sort"
        trailingIcon="arrow_right"
        disabled
      />
      <SettingsGroup
        settings={[
          {
            title: t('feedback.title'),
            leadingIcon: 'edit_note',
            onClick: async () => {
              navigate(routes.feedback);
            },
            trailing: (
              <MsIcon icon="arrow_right" style="dark:text-mercury-pinkish" />
            ),
          },
          {
            title: t('support.title'),
            leadingIcon: 'question_answer',
            onClick: async () => {
              navigate(routes.support);
            },
            trailing: (
              <MsIcon icon="arrow_right" style="dark:text-mercury-pinkish" />
            ),
          },
        ]}
      />
      <SettingsMenuCard
        title={t('legal.title')}
        onClick={async () => {
          navigate(routes.legal);
        }}
        trailingIcon="arrow_right"
      />
      <SettingsMenuCard title={t('quit')} onClick={exit} />
      <div className="flex grow flex-col justify-end text-comet text-sm tracking-tight leading-tight mb-4">
        Version {state.version}
      </div>
    </div>
  );
}

export default Settings;
