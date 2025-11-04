import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { useAutostart, useDesktopNotifications } from '../../hooks';
import { routes } from '../../router';
import { kvSet } from '../../kvStore';
import { useMainDispatch, useMainState } from '../../contexts';
import { useExit } from '../../state';
import { StateDispatch } from '../../types';
import { MsIcon, PageAnim, SettingsMenuCard, Switch } from '../../ui';
import { Account } from './account';
import { InfoData } from './info-data';
import SettingsGroup from './SettingsGroup';
import Logout from './Logout';

function Settings() {
  const { desktopNotifications, ipv6Support, backendFlags } = useMainState();

  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');
  const { exit } = useExit();
  const { enabled: autostartEnabled, toggle: toggleAutostart } = useAutostart();
  const toggleDNotifications = useDesktopNotifications();
  const showAntiCensorship = backendFlags.quic || backendFlags.domainFronting;

  const handleAutostartChanged = async () => {
    await toggleAutostart();
  };

  const handleIpv6Support = async () => {
    const switched = !ipv6Support;
    dispatch({ type: 'set-ipv6-support', enabled: switched });
    await kvSet('disable-ipv6', !switched);
  };

  return (
    <PageAnim className="xs:max-w-lg h-full flex flex-col mt-2 gap-6">
      <Account />
      <SettingsGroup
        settings={[
          {
            title: t('support.title'),
            leadingIcon: 'question_answer',
            onClick: () => navigate(routes.support),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
          {
            title: t('logs.title'),
            desc: t('logs.desc'),
            leadingIcon: 'sort',
            onClick: () => navigate(routes.logs),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
          {
            title: t('data-privacy', { ns: 'common' }),
            leadingIcon: 'encrypted',
            onClick: () => navigate(routes.dataPrivacy),
            trailing: <MsIcon icon="arrow_right" className="dark:text-white" />,
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('autostart.title'),
            desc: t('autostart.desc'),
            leadingIcon: 'computer',
            onClick: handleAutostartChanged,
            trailing: (
              <Switch
                checked={autostartEnabled}
                onChange={handleAutostartChanged}
              />
            ),
          },
          {
            title: t('ipv6-support.title'),
            desc: t('ipv6-support.desc'),
            leadingIcon: 'linear_scale',
            onClick: handleIpv6Support,
            trailing: (
              <Switch checked={ipv6Support} onChange={handleIpv6Support} />
            ),
          },
          {
            title: t('killswitch.title'),
            desc: t('killswitch.desc'),
            leadingIcon: 'power_settings_new',
            onClick: () => {
              /**/
            },
            trailing: (
              <Switch
                checked={true}
                onChange={() => {
                  /* */
                }}
                disabled
              />
            ),
            disabled: true,
          },
          showAntiCensorship && {
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
      <SettingsMenuCard
        title={t('legal.title')}
        onClick={() => navigate(routes.legal)}
        trailingIcon="arrow_right"
      />
      <Logout />
      <SettingsMenuCard title={t('quit')} onClick={exit} />
      <InfoData />
    </PageAnim>
  );
}

export default Settings;
