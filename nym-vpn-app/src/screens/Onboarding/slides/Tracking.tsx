import { useTranslation } from 'react-i18next';
import { StopTracking } from '../../../assets';

function Tracking() {
  const { t } = useTranslation('onboarding');
  return (
    <div className="flex flex-col items-center gap-4">
      <StopTracking className="h-full max-h-64 w-fit" />
      <h1 className="text-text-primary text-center text-2xl">
        {t('tracking.title')}
      </h1>
      <p className="text-text-secondary text-center text-base whitespace-pre-line">
        {t('tracking.description')}
      </p>
    </div>
  );
}

export default Tracking;
