import { Trans, useTranslation } from 'react-i18next';
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useMainDispatch, useMainState } from '../../../contexts';
import { Button, Link, PageAnim } from '../../../ui';
import { StateDispatch } from '../../../types';
import { MixnetParametersLearnMoreUrl } from '../../../constants';
import { MixnetTrafficConfigProvider, useMixnetTrafficConfig } from './context';
import { ContinuousTrafficCard } from './ContinuousTrafficCard';
import { MixingDelayCard } from './MixingDelayCard';
import { PerformanceCard } from './PerformanceCard';

function MixnetTuning() {
  const { t } = useTranslation('settings');

  const [loading, setLoading] = useState(false);

  const { state, dispatch, hasUnsavedSettings, hasSettingsOtherThanDefaults } =
    useMixnetTrafficConfig();
  const mainDispatch = useMainDispatch() as StateDispatch;

  const handleSaveCustomSettings = async () => {
    setLoading(true);
    try {
      await invoke('set_mixnet_traffic_config', { config: state });
      mainDispatch({
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
    <PageAnim className="h-full flex flex-col mt-2 pb-2 gap-6 justify-between select-none">
      <div className="flex flex-col gap-6">
        <p className="text-sm text-center text-iron dark:text-bombay whitespace-pre-line">
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
          color="malachite"
          disabled={!hasUnsavedSettings || loading}
          onClick={handleSaveCustomSettings}
          spinner={loading}
        >
          {t('mixnet-tuning.save-custom-settings')}
        </Button>
        {hasSettingsOtherThanDefaults && (
          <Button
            outline
            color="gray"
            onClick={() => dispatch({ type: 'restore-defaults' })}
            className="group border border-iron dark:border-bombay hover:ring-0! dark:hover:ring-0!"
          >
            <span className="text-black dark:text-white group-hover:text-black/50 dark:group-hover:text-white/80">
              {t('mixnet-tuning.restore-default-settings')}
            </span>
          </Button>
        )}
      </div>
    </PageAnim>
  );
}

function MixnetTuningWrapper() {
  const { mixnetTrafficConfig } = useMainState();

  return (
    <MixnetTrafficConfigProvider initialConfig={mixnetTrafficConfig}>
      <MixnetTuning />
    </MixnetTrafficConfigProvider>
  );
}

export default MixnetTuningWrapper;
