import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { useAutostart, useToast } from '../../hooks';
import { routes } from '../../router';
import { dispatch, useMainState } from '../../store';
import { useExit } from '../../state';
import { BetaPill, Button, MsIcon, PageAnim, Switch } from '../../ui';
import { AccountSettingRow } from './account';
import { InfoData } from './info-data';
import SettingsGroup from './SettingsGroup';

function Settings() {
  const { ipv6Support, allowLan, enableAdBlocking } = useMainState();

  const navigate = useNavigate();
  const { t } = useTranslation('settings');
  const { exit } = useExit();
  const { enabled: autostartEnabled, toggle: toggleAutostart } = useAutostart();
  const { add } = useToast();

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
      add({
        title: t('ipv6-support.errors.failed'),
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
      add({
        title: t('allow-lan.errors.failed'),
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
      add({
        title: t('ad-block.errors.failed'),
        type: 'error',
      });
    }
  };
  return (
    <PageAnim className="flex h-full flex-col gap-4">
      <AccountSettingRow />
      <SettingsGroup
        settings={[
          {
            title: t('support.title'),
            leadingIcon: 'chat_bubble',
            onClick: () => navigate(routes.support),
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('profiles.title', { ns: 'common' }),
            leadingIcon: 'local_fire_department',
            onClick: () => navigate(routes.profiles),
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
          },
          {
            title: t('killswitch.title'),
            desc: t('killswitch.desc'),
            leadingIcon: 'remove_moderator',
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
            leadingIcon: 'language',
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
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
          },
          {
            title: t('mixnet-tuning.title'),
            desc: t('mixnet-tuning.desc'),
            leadingIcon: 'visibility_off',
            onClick: () => navigate(routes.mixnetTuning),
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
          },
          {
            title: t('split-tunneling.title'),
            leadingIcon: 'call_split',
            onClick: () =>
              navigate(routes.splitTunneling, {
                state: { resetScroll: true },
              }),
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
          },
          {
            title: t('geo-exclusion.title'),
            titleTrailing: <BetaPill />,
            leadingIcon: 'public',
            onClick: () => navigate(routes.geoExclusion),
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
          },
          {
            title: t('anti-censorship.title', { ns: 'settings' }),
            leadingIcon: 'campaign',
            onClick: () => navigate(routes.antiCensorship),
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
          },
          {
            title: t('app-proxy.title'),
            desc: t('app-proxy.menu-desc'),
            leadingIcon: 'lan',
            onClick: () => navigate(routes.socks5),
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
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
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
          },
          {
            title: t('notifications.title'),
            leadingIcon: 'notifications',
            onClick: () => navigate(routes.notifications),
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
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
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
          },
        ]}
      />
      <SettingsGroup
        settings={[
          {
            title: t('legal.title'),
            onClick: () => navigate(routes.legal),
            trailing: (
              <MsIcon icon="chevron_right" className="text-text-primary" />
            ),
          },
        ]}
      />
      <Button variant="destructive" onClick={exit}>
        {t('quit')}
      </Button>
      <InfoData />
    </PageAnim>
  );
}

export default Settings;
