import { Trans, useTranslation } from 'react-i18next';
import { NoiseLine } from '../NoiseLine';

function Welcome() {
  const { t } = useTranslation('onboarding');

  return (
    <div className="flex h-full flex-col items-center justify-around">
      <NoiseLine />
      <div className="flex flex-col items-center gap-4 px-4">
        <h1 className="text-text-primary text-center text-2xl">
          {t('welcome.title')}
        </h1>
        <p className="text-text-secondary text-center text-sm whitespace-pre-line">
          <Trans
            i18nKey="welcome.subtitle"
            ns="onboarding"
            components={{ bold: <strong /> }}
          />
        </p>
      </div>
    </div>
  );
}

export default Welcome;
