import { useTranslation } from 'react-i18next';
import { Speed as SpeedAsset } from '../../../assets';

function Speed() {
  const { t } = useTranslation('onboarding');
  return (
    <div className="flex flex-col items-center gap-4">
      <SpeedAsset className="h-fit w-full max-w-72" />
      <h1 className="text-2xl text-baltic-sea dark:text-white">
        {t('speed.title')}
      </h1>
      <p className="text-center text-base text-iron dark:text-bombay">
        {t('speed.description')}
      </p>
    </div>
  );
}

export default Speed;
