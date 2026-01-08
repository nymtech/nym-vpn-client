import clsx from 'clsx';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { NymSplash } from '../../assets';
import { Button, ButtonText, PageAnim } from '../../ui';
import { useMainState } from '../../contexts';
import { NymVpnPricingUrl } from '../../constants';
import { routes } from '../../router';

function Login() {
  const { uiTheme } = useMainState();
  const { t } = useTranslation('login');
  const navigate = useNavigate();

  return (
    <PageAnim className="relative h-full flex flex-col justify-end items-center gap-6 select-none cursor-default">
      <NymSplash
        className={clsx('w-32', uiTheme === 'dark' ? 'fill-white' : 'fill-ash')}
      />
      <h1 className="text-2xl mt-12">{t('signup.title')}</h1>
      <div className="flex flex-col p-4">
        <div className="py-6 border-b border-bombay dark:border-iron">
          <h2>{t('signup.maximum-privacy.title')}</h2>
          <p className="mt-2 text-iron dark:text-bombay">
            {t('signup.maximum-privacy.description')}
          </p>
          <Button
            onClick={() => {
              openUrl(NymVpnPricingUrl);
              navigate(routes.login);
            }}
            className="mt-4"
          >
            {t('signup.create-account')}
          </Button>
        </div>
      </div>
      <div>
        <span>{t('signup.already-have-an-account.title')}</span>
        <ButtonText onClick={() => navigate(routes.login)} color="malachite">
          {t('signup.already-have-an-account.button')}
        </ButtonText>
      </div>
    </PageAnim>
  );
}

export default Login;
