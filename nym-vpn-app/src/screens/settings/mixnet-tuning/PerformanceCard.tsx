import { Separator } from '@base-ui-components/react';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import {
  CardDataRow,
  CardNew,
  CardNewBody,
  CardNewFooter,
  CardNewHeader,
} from '../../../ui';
import { useMixnetTrafficConfig } from './context';

export function PerformanceCard() {
  const { t } = useTranslation('settings');
  const { state } = useMixnetTrafficConfig();

  const speed = state.messageSendingAverageDelay;

  const [privacy, setPrivacy] = useState(0);

  useEffect(() => {
    invoke<number>('calculate_traffic_latency', { config: state }).then(
      (result) => setPrivacy(result),
    );
  }, [state]);

  return (
    <CardNew>
      <CardNewHeader>
        <p className="text-left text-sm text-iron dark:text-bombay whitespace-pre-line">
          {t('mixnet-tuning.performance.title')}
        </p>
      </CardNewHeader>
      <CardNewBody>
        <CardDataRow label={t('mixnet-tuning.performance.speed.title')}>
          <p className="text-malachite-moss dark:text-malachite font-medium">
            {t('mixnet-tuning.performance.speed.value', { value: speed })}
          </p>
        </CardDataRow>
        <Separator
          orientation="horizontal"
          className="w-full h-px bg-bombay dark:bg-iron"
        />
        <CardDataRow label={t('mixnet-tuning.performance.privacy.title')}>
          <p className="text-malachite-moss dark:text-malachite font-medium">
            {t('mixnet-tuning.performance.privacy.value', { value: privacy })}
          </p>
        </CardDataRow>
      </CardNewBody>

      <CardNewFooter>
        <p className="text-xs text-iron dark:text-bombay whitespace-pre-line">
          {t('mixnet-tuning.performance.footer')}
        </p>
      </CardNewFooter>
    </CardNew>
  );
}
