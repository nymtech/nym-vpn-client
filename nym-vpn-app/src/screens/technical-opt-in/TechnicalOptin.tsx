import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { PrivacyPolicyUrl } from '../../constants';
import { useToast } from '../../hooks';
import { kvSet } from '../../kvStore';
import { routes } from '../../router';
import { Button, Link, Switch } from '../../ui';
import { NymVpnTextLogo } from '../../assets';
import SettingsGroup from '../settings/SettingsGroup';
import { dispatch } from '../../store';
import { InteractiveCard } from '../home/InteractiveCard';
import TelemetryInfoDialog, {
  InfoButton,
  TelemetryInfoKind,
} from './TelemetryInfoDialog';

const defaultSentry = window._APP.defaultSentry;
const defaultNetstats = window._APP.defaultNetstats;

function TechnicalOptin() {
  const [monitoring, setMonitoring] = useState<boolean>(defaultSentry);
  const [netstats, setNetstats] = useState<boolean>(defaultNetstats);
  // last values the backend accepted, a rejected save snaps back to these
  const [savedMonitoring, setSavedMonitoring] =
    useState<boolean>(defaultSentry);
  const [savedNetstats, setSavedNetstats] = useState<boolean>(defaultNetstats);
  const [saving, setSaving] = useState(false);
  const [info, setInfo] = useState<TelemetryInfoKind | null>(null);
  const navigate = useNavigate();
  const { t } = useTranslation(['welcome', 'settings']);
  const { add } = useToast();

  const handleContinue = async () => {
    if (saving) {
      return;
    }
    setSaving(true);
    try {
      const monitoringApplied = await applyChoice('monitoring');
      const netstatsApplied = await applyChoice('netstats');
      if (!monitoringApplied || !netstatsApplied) {
        add({
          title: t('privacy.errors.failed', { ns: 'settings' }),
          type: 'error',
        });
        return;
      }
      dispatch({ type: 'set-technical-optin-seen', seen: true });
      kvSet('technical-optin-seen', true);
      navigate(routes.root);
    } finally {
      setSaving(false);
    }
  };

  const toggleMonitoring = () => {
    setMonitoring(!monitoring);
  };

  const toggleNetstats = () => {
    setNetstats(!netstats);
  };

  const choices = {
    monitoring: {
      checked: monitoring,
      saved: savedMonitoring,
      setChecked: setMonitoring,
      setSaved: setSavedMonitoring,
      enable: 'enable_sentry',
      disable: 'disable_sentry',
      action: 'set-monitoring',
      label: 'monitoring',
    },
    netstats: {
      checked: netstats,
      saved: savedNetstats,
      setChecked: setNetstats,
      setSaved: setSavedNetstats,
      enable: 'enable_netstats',
      disable: 'disable_netstats',
      action: 'set-network-stats',
      label: 'network stats',
    },
  } as const;

  const applyChoice = async (name: keyof typeof choices) => {
    const choice = choices[name];
    try {
      await invoke(choice.checked ? choice.enable : choice.disable);
      dispatch({ type: choice.action, enabled: choice.checked });
      choice.setSaved(choice.checked);
      return true;
    } catch (error) {
      console.error(`[technical-optin] ${choice.label} error`, error);
      choice.setChecked(choice.saved);
      return false;
    }
  };

  return (
    <InteractiveCard>
      <div className="flex flex-col items-center justify-center gap-4">
        <NymVpnTextLogo className="fill-text-primary h-6 w-24" />
        {/* Title & description */}
        <div className="space-y-2 text-center">
          <h1 className="text-text-primary text-2xl">{t('title')}</h1>
          <p className="text-text-tertiary text-sm whitespace-pre-line">
            {t('description')}
          </p>
          <Link
            color="iron"
            text={t('privacy-statement-link')}
            url={PrivacyPolicyUrl}
          />
        </div>

        {/* Opt-in toggles */}
        <SettingsGroup
          className="w-full"
          settings={[
            {
              title: t('connection.label'),
              titleTrailing: (
                <InfoButton
                  kind="connection"
                  label={t('connection.label')}
                  onOpen={setInfo}
                  data-testid="welcome-netstats-info"
                />
              ),
              desc: t('connection.desc'),
              leadingIcon: 'analytics',
              trailing: (
                <Switch
                  checked={netstats}
                  onChange={toggleNetstats}
                  disabled={saving}
                  data-testid="welcome-netstats-switch"
                />
              ),
              'data-testid': 'welcome-netstats-option',
            },
            {
              title: t('monitoring.label'),
              titleTrailing: (
                <InfoButton
                  kind="monitoring"
                  label={t('monitoring.label')}
                  onOpen={setInfo}
                  data-testid="welcome-monitoring-info"
                />
              ),
              desc: t('monitoring.desc'),
              leadingIcon: 'bug_report',
              trailing: (
                <Switch
                  checked={monitoring}
                  onChange={toggleMonitoring}
                  disabled={saving}
                  data-testid="welcome-monitoring-switch"
                />
              ),
              'data-testid': 'welcome-monitoring-option',
            },
          ]}
          data-testid="welcome-optin-group"
        />
        <Button onClick={handleContinue} disabled={saving}>
          {t('continue-button')}
        </Button>
      </div>

      <TelemetryInfoDialog kind={info} onClose={() => setInfo(null)} />
    </InteractiveCard>
  );
}

export default TechnicalOptin;
