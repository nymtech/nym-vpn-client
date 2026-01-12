import clsx from 'clsx';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Trans, useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { NymSplash } from '../../assets';
import { Button, ButtonText, Link, MsIcon, PageAnim } from '../../ui';
import { useMainState } from '../../contexts';
import { NymVpnPricingUrl, PrivacyPolicyUrl, ToSUrl } from '../../constants';
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
        <div className="py-6">
          <h2>{t('signup.maximum-privacy.title')}</h2>
          <p className="mt-2 text-iron dark:text-bombay whitespace-pre-line">
            {t('signup.maximum-privacy.description')}
          </p>
          <Button
            onClick={() => {
              openUrl(NymVpnPricingUrl);
              navigate(routes.login);
            }}
            className="mt-4"
          >
            <div className="flex items-center gap-2">
              {t('signup.create-account')} <MsIcon icon="open_in_new" />
            </div>
          </Button>
        </div>

        <div className="flex flex-row justify-center items-center">
          <span className="dark:text-white truncate">
            {t('signup.already-have-an-account.title')}
          </span>
          <ButtonText onClick={() => navigate(routes.login)} color="malachite">
            {t('signup.already-have-an-account.button')}
          </ButtonText>
        </div>
      </div>
      <p
        className="text-xs text-center text-iron dark:text-bombay w-80"
        data-testid="welcome-tos-notice"
      >
        <Trans
          i18nKey="tos-notice"
          ns="welcome"
          components={{
            tosLink: (
              <Link
                color="primary"
                text={t('tos', { ns: 'common' })}
                url={ToSUrl}
                textClassName="underline-offset-2"
                data-testid="welcome-tos-link"
              />
            ),
            privacyLink: (
              <Link
                color="primary"
                text={t('privacy-statement', { ns: 'common' })}
                url={PrivacyPolicyUrl}
                textClassName="underline-offset-2"
                data-testid="welcome-privacy-link"
              />
            ),
          }}
        />
      </p>
    </PageAnim>
  );
}

export default Login;
