import { Separator } from '@base-ui-components/react';
import { Trans, useTranslation } from 'react-i18next';
import { SettingsMenuCardBig } from '../../../ui';

export function PerformanceCard() {
  const { t } = useTranslation('settings');

  return (
    <SettingsMenuCardBig
      header={
        <div className="p-5 pb-0 w-full flex flex-row items-start justify-start">
          <p className=" text-left text-sm text-iron dark:text-bombay whitespace-pre-line">
            {t('mixnet-tuning.performance.title')}
          </p>
        </div>
      }
      footer={
        <div className="p-5 pt-0 w-full">
          <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
            {t('mixnet-tuning.performance.footer')}
          </p>
        </div>
      }
    >
      <div className="flex flex-col justify-center items-start gap-3">
        <DataRow label={t('mixnet-tuning.performance.speed.title')}>
          <div className="flex gap-1 items-center overflow-hidden select-none">
            <p className="text-malachite-moss dark:text-malachite font-medium">
              <Trans
                i18nKey="mixnet-tuning.performance.speed.value"
                ns="settings"
                components={{
                  value: <span>{1}</span>,
                }}
              />
            </p>
          </div>
        </DataRow>
        <Separator
          orientation="horizontal"
          className="w-full h-px bg-bombay dark:bg-iron"
        />
        <DataRow label={t('mixnet-tuning.performance.privacy.title')}>
          <div className="flex gap-1 items-center overflow-hidden select-none">
            <p className="text-malachite-moss dark:text-malachite font-medium">
              <Trans
                i18nKey="mixnet-tuning.performance.privacy.value"
                ns="settings"
                components={{
                  value: <span>{700}</span>,
                }}
              />
            </p>
          </div>
        </DataRow>
      </div>
    </SettingsMenuCardBig>
  );
}

const DataRow = ({
  children,
  label,
}: {
  children: React.ReactNode;
  label: string;
}) => (
  <div className="w-full flex justify-between items-center">
    <p className="text-iron dark:text-bombay truncate select-none">{label}</p>
    <div className="flex flex-nowrap items-center gap-2 overflow-hidden">
      {children}
    </div>
  </div>
);
