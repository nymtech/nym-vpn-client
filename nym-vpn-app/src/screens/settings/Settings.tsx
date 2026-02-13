import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { useAutostart, useDesktopNotifications } from '../../hooks';
import { routes } from '../../router';
import { useMainDispatch, useMainState } from '../../contexts';
import { useExit } from '../../state';
import { StateDispatch } from '../../types';
import { MsIcon, PageAnim, SettingsMenuCard, Switch } from '../../ui';
import { AccountSettingRow } from './account';
import { InfoData } from './info-data';
import SettingsGroup from './SettingsGroup';
import Logout from './Logout';

function Settings() {
  const { desktopNotifications, ipv6Support, allowLan, backendFlags } =
    useMainState();

  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');
  const { exit } = useExit();
  const { enabled: autostartEnabled, toggle: toggleAutostart } = useAutostart();
  const toggleDNotifications = useDesktopNotifications();

  const handleAutostartChanged = async () => {
    await toggleAutostart();
  };

  const handleIpv6Support = async () => {
    const switched = !ipv6Support;
    try {
      await invoke('set_no_ipv6', { enabled: !switched });
      dispatch({ type: 'set-ipv6-support', enabled: switched });
    } catch {
      /* TODO */
    }
  };

  const handleAllowLan = async () => {
    const switched = !allowLan;
    try {
      await invoke('set_allow_lan', { enabled: switched });
      dispatch({ type: 'set-allow-lan', enabled: switched });
    } catch {
      /* TODO */
    }
  };

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      <AccountSettingRow />
      <SettingsGroup
        settings={[
          {
            title: t('support.title'),
            leadingIcon: 'question_answer',
            onClick: () => navigate(routes.support),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('killswitch.title'),
            desc: t('killswitch.desc'),
            leadingIcon: 'power',
          },
          {
            title: t('ipv6-support.title'),
            desc: t('ipv6-support.desc'),
            leadingIcon: 'add_moderator',
            onClick: handleIpv6Support,
            trailing: (
              <Switch checked={ipv6Support} onChange={handleIpv6Support} />
            ),
          },
          {
            title: t('allow-lan.title'),
            desc: t('allow-lan.desc'),
            leadingIcon: 'lan',
            onClick: handleAllowLan,
            trailing: <Switch checked={allowLan} onChange={handleAllowLan} />,
          },
          {
            title: t('dns.title'),
            leadingIcon: 'dns',
            onClick: () => navigate(routes.dns),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
          backendFlags.mixnetTuning && {
            title: t('mixnet-tuning.title'),
            desc: t('mixnet-tuning.desc'),
            leadingIcon: 'visibility_off',
            onClick: () => navigate(routes.mixnetTuning),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
          {
            title: t('anti-censorship.title', { ns: 'settings' }),
            leadingIcon: 'campaign',
            onClick: () => navigate(routes.antiCensorship),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
          {
            title: t('app-proxy.title'),
            desc: t('app-proxy.menu-desc'),
            leadingIcon: 'lan',
            onClick: () => navigate(routes.socks5),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('autostart.title'),
            desc: t('autostart.desc'),
            leadingIcon: 'rocket_launch',
            onClick: handleAutostartChanged,
            trailing: (
              <Switch
                checked={autostartEnabled}
                onChange={handleAutostartChanged}
              />
            ),
          },
          {
            title: t('appearance', { ns: 'common' }),
            leadingIcon: 'view_comfy',
            onClick: () => navigate(routes.appearance),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
          {
            title: t('notifications.title'),
            leadingIcon: 'notifications',
            onClick: toggleDNotifications,
            trailing: (
              <Switch
                checked={desktopNotifications}
                onChange={toggleDNotifications}
              />
            ),
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('logs.title'),
            leadingIcon: 'notes',
            onClick: () => navigate(routes.logs),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
          {
            title: t('data-privacy', { ns: 'common' }),
            leadingIcon: 'privacy_tip',
            onClick: () => navigate(routes.dataPrivacy),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('legal.title'),
            onClick: () => navigate(routes.legal),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
        ]}
      />
      <Logout />
      <SettingsMenuCard title={t('quit')} onClick={exit} />
      <InfoData />
    </PageAnim>
  );
}

export default Settings;
