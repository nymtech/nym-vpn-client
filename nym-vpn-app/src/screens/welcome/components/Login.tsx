import { useTranslation } from 'react-i18next';
import { ButtonNew, MsIcon } from '../../../ui';

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
        <ButtonNew onClick={() => undefined} variant="outlined">
          {t('login.login-social-button')}
          <MsIcon icon="open_in_new" className="text-[18px]" />
        </ButtonNew>
        <p className="text-bombay text-center text-xs leading-5">
          {t('login.login-social-button-disclaimer')}
        </p>
      </div>
    </div>
  );
}
