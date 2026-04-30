import { useTranslation } from 'react-i18next';
import { ButtonNew } from '../../../ui';

type Props = {
  onSignup: () => void;
  onLogin: () => void;
};

export function Welcome({ onSignup, onLogin }: Props) {
  const { t } = useTranslation('login');
  return (
    <div className="flex flex-col items-center gap-6 h-full justify-between">
      <div className="flex flex-col items-center gap-2">
        <h1 className="text-2xl font-medium tracking-tight text-baltic-sea dark:text-white">
          {t('welcome.title')}
        </h1>
        <p className="text-sm text-bombay text-center w-[281px]">
          {t('welcome.description')}
        </p>
      </div>
      <div className="flex flex-col gap-3 w-full">
        <ButtonNew onClick={onSignup}>{t('welcome.signup-button')}</ButtonNew>
        <ButtonNew onClick={onLogin}>{t('welcome.login-button')}</ButtonNew>
        <p className="text-xs text-bombay text-center leading-5">
          {t('welcome.terms-of-service')}
        </p>
      </div>
    </div>
  );
}
