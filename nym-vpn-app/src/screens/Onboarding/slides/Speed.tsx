import { Trans, useTranslation } from 'react-i18next';
import { Speed as SpeedAsset } from '../../../assets';

function Speed() {
  const { t } = useTranslation('onboarding');
  return (
    <div className="flex flex-col items-center gap-4">
      <SpeedAsset className="h-full max-h-64 w-fit" />
      <h1 className="text-text-primary text-center text-2xl whitespace-pre-line">
        {t('speed.title')}
      </h1>
      <p className="text-text-secondary text-center text-base whitespace-pre-line">
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
