import { Trans, useTranslation } from 'react-i18next';
import { Speed as SpeedAsset } from '../../../assets';

function Speed() {
  const { t } = useTranslation('onboarding');
  return (
    <div className="flex flex-col items-center gap-4">
      <SpeedAsset className="h-full max-h-72 w-fit" />
      <h1 className="text-2xl text-center whitespace-pre-line text-baltic-sea dark:text-white">
        {t('speed.title')}
      </h1>
      <p className="text-center text-base whitespace-pre-line text-iron dark:text-bombay">
        <Trans
          i18nKey="speed.description"
          ns="onboarding"
          components={{ bold: <strong /> }}
        />
      </p>
    </div>
  );
}

export default Speed;
