import { Trans, useTranslation } from 'react-i18next';
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { dispatch } from '../../../store';
import { Button, Link, PageAnim } from '../../../ui';
import { MixnetParametersLearnMoreUrl } from '../../../constants';
import { MixnetTrafficConfigProvider, useMixnetTrafficConfig } from './context';
import { ContinuousTrafficCard } from './ContinuousTrafficCard';
import { BackgroundCoverCard } from './BackgroundCoverCard';
import { MixingDelayCard } from './MixingDelayCard';
import { PerformanceCard } from './PerformanceCard';

function MixnetTuning() {
  const { t } = useTranslation('settings');

  const [loading, setLoading] = useState(false);

  const {
    state,
    restoreDefaults,
    hasUnsavedSettings,
    hasSettingsOtherThanDefaults,
  } = useMixnetTrafficConfig();
  const handleSaveCustomSettings = async () => {
    setLoading(true);
    try {
      await invoke('set_mixnet_traffic_config', { config: state });
      dispatch({
        type: 'set-mixnet-traffic-config',
        config: {
          ...state,
        },
      });
    } catch (error) {
      console.error('[handleSaveCustomSettings] error', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <PageAnim className="mt-2 flex h-full flex-col justify-between gap-6 pb-2 select-none">
      <div className="flex flex-col gap-6">
        <p className="text-text-secondary text-sm whitespace-pre-line">
          <Trans
            i18nKey="mixnet-tuning.top-description"
            ns="settings"
            components={{
              strong: <strong className="font-bold" />,
            }}
          />
        </p>

        <PerformanceCard />
        <ContinuousTrafficCard />
        <BackgroundCoverCard />
        <MixingDelayCard />

        <Link
          className="w-fit text-sm"
          text={t('mixnet-tuning.learn-more')}
          url={MixnetParametersLearnMoreUrl}
          color="primary"
          icon
        />
      </div>

      <div className="flex flex-col gap-2 justify-self-end">
        <Button
          variant="primary"
          disabled={!hasUnsavedSettings || loading}
          onClick={handleSaveCustomSettings}
          loading={loading}
        >
          {t('mixnet-tuning.save-custom-settings')}
        </Button>

        <Button
          variant="outlined"
          onClick={restoreDefaults}
          disabled={!hasSettingsOtherThanDefaults}
        >
          {t('mixnet-tuning.restore-default-settings')}
        </Button>
      </div>
    </PageAnim>
  );
}

function MixnetTuningWrapper() {
  return (
    <MixnetTrafficConfigProvider>
      <MixnetTuning />
    </MixnetTrafficConfigProvider>
  );
}

export default MixnetTuningWrapper;
