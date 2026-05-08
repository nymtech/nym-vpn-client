import { useTranslation } from 'react-i18next';
import { ButtonNew, MsIcon } from '../../../ui';

type Props = {
  onPassphrase: () => void;
};

export function Login({ onPassphrase }: Props) {
  const { t } = useTranslation('login');
  return (
    <div className="flex flex-col items-center gap-6 h-full justify-between">
      <div className="flex flex-col items-center gap-2">
        <h1 className="text-2xl font-medium tracking-tight text-text-primary">
          {t('login.title')}
        </h1>
        <p className="text-sm text-bombay text-center w-[281px]">
          {t('login.description')}
        </p>
      </div>
      <div className="flex flex-col gap-3 w-full">
        <ButtonNew onClick={onPassphrase}>
          {t('login.login-24-words-button')}
        </ButtonNew>
        <ButtonNew onClick={() => undefined} variant="outlined">
          {t('login.login-social-button')}
          <MsIcon icon="open_in_new" className="text-[18px]" />
        </ButtonNew>
        <p className="text-xs text-bombay text-center leading-5">
          {t('login.login-social-button-disclaimer')}
        </p>
      </div>
    </div>
  );
}
