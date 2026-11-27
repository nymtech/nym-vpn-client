import { ReactNode, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { useAutostart, useToast } from '../../hooks';
import { routes } from '../../router';
import { dispatch, useMainState } from '../../store';
import { useExit } from '../../state';
import {
  BetaPill,
  Button,
  MsIcon,
  PageAnim,
  Switch,
  TextInput,
} from '../../ui';
import { AccountSettingRow, LogoutButton } from './account';
import { InfoData } from './info-data';
import SettingsGroup, { Setting } from './SettingsGroup';

function Settings() {
  const {
    ipv6Support,
    allowLan,
    enableAdBlocking,
    enableConflictDetection,
    account,
    accountState,
  } = useMainState();

  const navigate = useNavigate();
  const { t } = useTranslation('settings');
  const { exit } = useExit();
  const { enabled: autostartEnabled, toggle: toggleAutostart } = useAutostart();
  const { add } = useToast();
  const [query, setQuery] = useState('');

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

  const handleConflictDetection = async () => {
    const switched = !enableConflictDetection;
    try {
      await invoke('set_conflict_detection', { enabled: switched });
      dispatch({ type: 'set-enable-conflict-detection', enabled: switched });
    } catch (error) {
      console.error('[settings] conflict detection error', error);
      add({
        title: t('conflict-detection.errors.failed'),
        type: 'error',
      });
    }
  };
  const chevron = <MsIcon icon="chevron_right" className="text-text-primary" />;

  const sections: { key: string; title: string; settings: Setting[] }[] = [
    {
      key: 'protection',
      title: t('sections.protection'),
      settings: [
        {
          title: t('ad-block.title'),
          desc: enableAdBlocking
            ? t('ad-block.desc-on')
            : t('ad-block.desc-off'),
          leadingIcon: 'gpp_maybe',
          onClick: handleAdBlock,
          trailing: (
            <Switch checked={enableAdBlocking} onChange={handleAdBlock} />
          ),
        },
        {
          title: t('killswitch.title'),
          desc: t('killswitch.desc'),
          leadingIcon: 'remove_moderator',
        },
        {
          title: t('mixnet-tuning.title'),
          desc: t('mixnet-tuning.desc'),
          leadingIcon: 'visibility_off',
          onClick: () => navigate(routes.mixnetTuning),
          trailing: chevron,
        },
      ],
    },
    {
      key: 'connection',
      title: t('sections.connection'),
      settings: [
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
          title: t('conflict-detection.title'),
          desc: t('conflict-detection.desc'),
          leadingIcon: 'troubleshoot',
          onClick: handleConflictDetection,
          trailing: (
            <Switch
              checked={enableConflictDetection}
              onChange={handleConflictDetection}
            />
          ),
        },
        {
          title: t('anti-censorship.title', { ns: 'settings' }),
          leadingIcon: 'campaign',
          onClick: () => navigate(routes.antiCensorship),
          trailing: chevron,
        },
        {
          title: t('profiles.title', { ns: 'common' }),
          desc: t('profiles.menu-desc', { ns: 'common' }),
          leadingIcon: 'local_fire_department',
          onClick: () => navigate(routes.profiles),
          trailing: chevron,
        },
        {
          title: t('dns.title'),
          leadingIcon: 'dns',
          onClick: () => navigate(routes.dns),
          trailing: chevron,
        },
      ],
    },
    {
      key: 'routing',
      title: t('sections.routing'),
      settings: [
        {
          title: t('split-tunneling.title'),
          leadingIcon: 'call_split',
          onClick: () =>
            navigate(routes.splitTunneling, {
              state: { resetScroll: true },
            }),
          trailing: chevron,
        },
        {
          title: t('geo-exclusion.title'),
          titleTrailing: <BetaPill />,
          desc: t('geo-exclusion.menu-desc'),
          leadingIcon: 'public',
          onClick: () => navigate(routes.geoExclusion),
          trailing: chevron,
        },
        {
          title: t('app-proxy.title'),
          titleTrailing: <BetaPill />,
          desc: t('app-proxy.menu-desc'),
          leadingIcon: 'alt_route',
          onClick: () => navigate(routes.socks5),
          trailing: chevron,
        },
      ],
    },
    {
      key: 'app',
      title: t('sections.app'),
      settings: [
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
          trailing: chevron,
        },
        {
          title: t('notifications.title'),
          leadingIcon: 'notifications',
          onClick: () => navigate(routes.notifications),
          trailing: chevron,
        },
      ],
    },
    {
      key: 'help-data',
      title: t('sections.help-data'),
      settings: [
        {
          title: t('support.title'),
          leadingIcon: 'chat_bubble',
          onClick: () => navigate(routes.support),
          trailing: chevron,
        },
        {
          title: t('privacy.title', { ns: 'settings' }),
          leadingIcon: 'privacy_tip',
          onClick: () =>
            navigate(routes.dataPrivacy, { state: { resetScroll: true } }),
          trailing: chevron,
        },
        {
          title: t('legal.title'),
          leadingIcon: 'gavel',
          onClick: () => navigate(routes.legal),
          trailing: chevron,
        },
      ],
    },
  ];

  // ignore case and accents, so "ecran" finds "Écran"
  const fold = (text: string) =>
    text.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();
  const search = fold(query.trim());
  const matches = (text: string) => fold(text).includes(search);
  // a row matches on its title or its plain-text description
  const rowMatches = (s: Setting) =>
    matches(s.title) || (typeof s.desc === 'string' && matches(s.desc));
  // while searching, keep only matching rows under their section header;
  // a match on the header keeps the whole section
  const visibleSections = sections
    .map((section) => ({
      ...section,
      settings:
        !search || matches(section.title)
          ? section.settings
          : section.settings.filter(rowMatches),
    }))
    .filter((section) => section.settings.length > 0);
  // the account section matches on its heading and on the titles it shows
  const accountTitles = account
    ? [
        t('account.your-account'),
        ...(accountState === 'no-subscription'
          ? [t('account.choose-plan')]
          : []),
      ]
    : [t('account.get-started')];
  const showAccount =
    !search || [t('sections.account'), ...accountTitles].some(matches);
  // Log out and Quit are searchable too, like any other item
  const showLogout = !!account && (!search || matches(t('account.logout')));
  const showQuit = !search || matches(t('quit'));
  const noResults =
    !showAccount && visibleSections.length === 0 && !showLogout && !showQuit;

  return (
    <PageAnim className="flex h-full flex-col gap-4">
      <TextInput
        value={query}
        onChange={setQuery}
        placeholder={t('search.placeholder')}
        leftIcon="search"
        spellCheck={false}
        clearable
      />
      {showAccount && (
        <SettingsSection title={t('sections.account')}>
          <AccountSettingRow />
        </SettingsSection>
      )}
      {visibleSections.map((section) => (
        <SettingsSection key={section.key} title={section.title}>
          <SettingsGroup settings={section.settings} />
        </SettingsSection>
      ))}
      {noResults && (
        <p className="text-text-secondary px-5 text-sm select-none">
          {t('no-results-found.title', { ns: 'node-location' })}
        </p>
      )}
      {showLogout && <LogoutButton variant="grey" />}
      {showQuit && (
        <Button variant="destructive" onClick={exit}>
          {t('quit')}
        </Button>
      )}
      <InfoData />
    </PageAnim>
  );
}

function SettingsSection({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="flex flex-col gap-2">
      <h2 className="text-text-secondary px-5 text-sm select-none">{title}</h2>
      {children}
    </section>
  );
}

export default Settings;
