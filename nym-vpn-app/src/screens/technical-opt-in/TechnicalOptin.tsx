import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { Description, Field, Label } from '@headlessui/react';
import clsx from 'clsx';
import { AnonNetworkStatsUrl, SentryPrivacyPolicyUrl } from '../../constants';
import { kvSet } from '../../kvStore';
import { routes } from '../../router';
import { Button, Link, Switch } from '../../ui';
import { NymVpnTextLogo } from '../../assets';
import { dispatch } from '../../store';
import { InteractiveCard } from '../home/InteractiveCard';

const defaultSentry = window._APP.defaultSentry;
const defaultNetstats = window._APP.defaultNetstats;

function TechnicalOptin() {
  const [monitoring, setMonitoring] = useState<boolean>(defaultSentry);
  const [netstats, setNetstats] = useState<boolean>(defaultNetstats);
  const navigate = useNavigate();
  const { t } = useTranslation(['welcome', 'settings']);

  const handleContinue = () => {
    initMonitoring();
    initNetworkStats();
    dispatch({ type: 'set-technical-optin-seen', seen: true });
    kvSet('technical-optin-seen', true);
    navigate(routes.root);
  };

  const initMonitoring = async () => {
    dispatch({ type: 'set-monitoring', enabled: monitoring });
    // sentry is disabled by default
    if (monitoring) {
      try {
        await invoke('enable_sentry');
      } catch {}
    }
  };

  const initNetworkStats = async () => {
    dispatch({ type: 'set-network-stats', enabled: netstats });
    try {
      if (netstats) {
        await invoke('enable_netstats');
      } else {
        await invoke('disable_netstats');
      }
    } catch {}
  };

  return (
    <InteractiveCard>
      <div className="flex flex-col items-center justify-center gap-4">
        <NymVpnTextLogo className="fill-text-primary h-6 w-24" />
        {/* Title & description */}
        <div className="space-y-2 text-center">
          <h1 className="text-text-primary text-2xl">{t('title')}</h1>
          <p className="text-text-tertiary text-sm">{t('description')}</p>
        </div>

        <div className="w-full" data-testid="welcome-netstats-group">
          {[
            {
              id: 'welcome-netstats',
              label: t('anonymous-network-stats'),
              helpText: t('privacy.network-stats.link', { ns: 'settings' }),
              helpUrl: AnonNetworkStatsUrl,
              icon: 'analytics',
              checked: netstats,
              onChange: setNetstats,
            },
            {
              id: 'welcome-monitoring',
              label: t('error-and-crash-reports'),
              helpText: t('privacy.error-monitoring.link', { ns: 'settings' }),
              helpUrl: SentryPrivacyPolicyUrl,
              icon: 'bug_report',
              checked: monitoring,
              onChange: setMonitoring,
            },
          ].map(
            (
              { id, label, helpText, helpUrl, icon, checked, onChange },
              index,
            ) => (
              <Field
                key={id}
                className={clsx(
                  'bg-surface-elev hover:bg-surface-hair flex min-h-16 items-center gap-4 px-5 py-2 transition duration-75',
                  index === 0
                    ? 'rounded-t-2xl'
                    : 'border-surface-bg rounded-b-2xl border-t',
                )}
                data-testid={`${id}-option`}
              >
                <span
                  aria-hidden="true"
                  className="font-icon text-text-secondary text-2xl select-none"
                >
                  {icon}
                </span>
                <div className="flex min-w-4 flex-1 flex-col justify-center">
                  <Label
                    as="span"
                    className="text-text-primary truncate text-base select-none"
                  >
                    {label}
                  </Label>
                  <Description
                    as="div"
                    className="text-text-secondary truncate text-sm select-none"
                  >
                    <Link color="iron" text={helpText} url={helpUrl} />
                  </Description>
                </div>
                <Switch
                  checked={checked}
                  onChange={onChange}
                  data-testid={`${id}-switch`}
                />
              </Field>
            ),
          )}
        </div>
        <Button onClick={handleContinue}>{t('continue-button')}</Button>
      </div>
    </InteractiveCard>
  );
}

export default TechnicalOptin;
