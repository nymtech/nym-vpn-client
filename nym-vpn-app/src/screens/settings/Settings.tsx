import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { type } from '@tauri-apps/plugin-os';
import { useAutostart, useDesktopNotifications } from '../../hooks';
import { routes } from '../../router';
import { useInAppNotify, useMainDispatch, useMainState } from '../../contexts';
import { useExit } from '../../state';
import { StateDispatch } from '../../types';
import { MsIcon, PageAnim, SettingsMenuCard, Switch } from '../../ui';
import { AccountSettingRow } from './account';
import { InfoData } from './info-data';
import SettingsGroup from './SettingsGroup';

function Settings() {
  const {
    desktopNotifications,
    ipv6Support,
    allowLan,
    enableLewesProtocol,
    enableAdBlocking,
    backendFlags,
  } = useMainState();

  const navigate = useNavigate();
  const dispatch = useMainDispatch() as StateDispatch;
  const { t } = useTranslation('settings');
  const { exit } = useExit();
  const { enabled: autostartEnabled, toggle: toggleAutostart } = useAutostart();
  const toggleDNotifications = useDesktopNotifications();
  const { push } = useInAppNotify();

  const os = type();

  const handleAutostartChanged = async () => {
    await toggleAutostart();
  };

  const handleIpv6Support = async () => {
    const switched = !ipv6Support;
    try {
      await invoke('set_no_ipv6', { enabled: !switched });
      dispatch({ type: 'set-ipv6-support', enabled: switched });
    } catch (error) {
      console.error('[settings] IPv6 support error', error);
      push({
        message: t('ipv6-support.errors.failed'),
        close: true,
        type: 'error',
      });
    }
  };

  const handleAllowLan = async () => {
    const switched = !allowLan;
    try {
      await invoke('set_allow_lan', { enabled: switched });
      dispatch({ type: 'set-allow-lan', enabled: switched });
    } catch (error) {
      console.error('[settings] allow lan error', error);
      push({
        message: t('allow-lan.errors.failed'),
        close: true,
        type: 'error',
      });
    }
  };

  const handleLewesProtocol = async () => {
    const switched = !enableLewesProtocol;
    try {
      await invoke('set_enable_lewes_protocol', { enabled: switched });
      dispatch({ type: 'set-enable-lewes-protocol', enabled: switched });
    } catch (error) {
      console.error('[settings] lewes protocol error', error);
      push({
        message: t('lewes.errors.failed'),
        close: true,
        type: 'error',
      });
    }
  };

  const handleAdBlock = async () => {
    const switched = !enableAdBlocking;
    try {
      await invoke('set_ad_block', { enabled: switched });
      dispatch({ type: 'set-enable-ad-blocking', enabled: switched });
    } catch (error) {
      console.error('[settings] ad block error', error);
      push({
        message: t('ad-block.errors.failed'),
        close: true,
        type: 'error',
      });
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
            title: t('ad-block.title'),
            desc: t('ad-block.desc'),
            leadingIcon: 'gpp_maybe',
            onClick: handleAdBlock,
            trailing: (
              <Switch checked={enableAdBlocking} onChange={handleAdBlock} />
            ),
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
            title: t('lewes.title'),
            desc: enableLewesProtocol ? t('lewes.desc.on') : t('lewes.desc.off'),
            leadingIcon: 'matter',
            onClick: handleLewesProtocol,
            trailing: (
              <Switch
                checked={enableLewesProtocol}
                onChange={handleLewesProtocol}
              />
            ),
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
          ...(os === 'windows'
            ? [
              {
                title: t('split-tunneling.title'),
                leadingIcon: 'call_split',
                onClick: () =>
                  navigate(routes.splitTunneling, {
                    state: { resetScroll: true },
                  }),
                trailing: (
                  <MsIcon icon="arrow_right" className="dark:text-white" />
                ),
              },
            ]
            : []),
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
            title: t('privacy.title', { ns: 'settings' }),
            leadingIcon: 'privacy_tip',
            onClick: () =>
              navigate(routes.dataPrivacy, { state: { resetScroll: true } }),
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
      <SettingsMenuCard title={t('quit')} onClick={exit} />
      <InfoData />
    </PageAnim>
  );
}

export default Settings;
