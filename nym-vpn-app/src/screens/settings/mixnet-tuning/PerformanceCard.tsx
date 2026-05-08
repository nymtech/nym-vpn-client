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
  const { state, continuousItems } = useMixnetTrafficConfig();

  const speed = continuousItems.find(
    (item) => item.value === state.messageSendingAverageDelay,
  )?.label;

  const [privacy, setPrivacy] = useState(0);

  useEffect(() => {
    invoke<number>('calculate_traffic_latency', { config: state }).then(
      (result) => setPrivacy(result),
    );
  }, [state]);

  return (
    <CardNew>
      <CardNewHeader>
        <p className="text-text-secondary text-left text-sm whitespace-pre-line">
          {t('mixnet-tuning.performance.title')}
        </p>
      </CardNewHeader>
      <CardNewBody>
        <CardDataRow label={t('mixnet-tuning.performance.speed.title')}>
          <p className="text-primary font-medium">
            {t('mixnet-tuning.performance.speed.value', { value: speed })}
          </p>
        </CardDataRow>
        <Separator
          orientation="horizontal"
          className="bg-bombay dark:bg-iron h-px w-full"
        />
        <CardDataRow label={t('mixnet-tuning.performance.privacy.title')}>
          <p className="text-primary font-medium">
            {t('mixnet-tuning.performance.privacy.value', { value: privacy })}
          </p>
        </CardDataRow>
      </CardNewBody>

      <CardNewFooter>
        <p className="text-text-secondary text-xs whitespace-pre-line">
          {t('mixnet-tuning.performance.footer')}
        </p>
      </CardNewFooter>
    </CardNew>
  );
}
