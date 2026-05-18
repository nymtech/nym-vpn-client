import { useTranslation } from 'react-i18next';
import { ButtonNew } from '../../../ui';
import { PrivyButton } from '../../../components/index';

type Props = {
  onPassphrase: () => void;
};

export function Login({ onPassphrase }: Props) {
  const { t } = useTranslation('login');
  return (
    <div className="flex h-full flex-col items-center justify-between gap-6">
      <div className="flex flex-col items-center gap-2">
        <h1 className="text-text-primary text-2xl font-medium tracking-tight">
          {t('login.title')}
        </h1>
      </div>
      <div className="flex w-full flex-col gap-3">
        <ButtonNew onClick={onPassphrase}>
          {t('login.login-24-words-button')}
        </ButtonNew>
        <PrivyButton label={t('login.login-social-button')} />
        <p className="text-bombay text-center text-xs leading-5">
          {t('login.login-social-button-disclaimer')}
        </p>
      </div>
    </div>
  );
}
