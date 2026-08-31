import { Trans, useTranslation } from 'react-i18next';
import { AroundGlow } from '../../../assets/onboarding';

function Around() {
  const { t } = useTranslation('onboarding');

  return (
    <div className="flex flex-col items-center gap-2 px-4">
      <h1 className="text-text-primary text-center text-2xl uppercase">
        {t('around.title')}
      </h1>
      <img
        src={AroundGlow}
        alt=""
        className="h-full max-h-60 w-auto max-w-full shrink-0 object-contain"
      />
      <p className="text-text-secondary text-center text-sm whitespace-pre-line">
        <Trans
          i18nKey="around.description"
          ns="onboarding"
          components={{ bold: <strong /> }}
        />
      </p>
    </div>
  );
}

export default Around;
