import { Trans, useTranslation } from 'react-i18next';
import { Welcome as WelcomeAsset } from '../../../assets';

function Welcome() {
  const { t } = useTranslation('onboarding');
  return (
    <div className="flex flex-col items-center gap-4">
      <WelcomeAsset className="h-full max-h-64 w-fit" />
      <h1 className="text-2xl text-baltic-sea dark:text-white">
        {t('welcome.title')}
      </h1>
      <p className="text-center text-sm whitespace-pre-line text-iron dark:text-bombay">
        <Trans
          i18nKey="welcome.description"
          ns="onboarding"
          components={{ large: <span className="text-base!" /> }}
        />
      </p>
    </div>
  );
}

export default Welcome;
