import clsx from 'clsx';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Trans, useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { NymSplash } from '../../assets';
import { Button, Link, MsIcon, PageAnim } from '../../ui';
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
      <h1 className="text-2xl mt-12">{t('privy.title')}</h1>
      <div className="flex flex-col p-4">
        <div className="py-6 border-b border-bombay dark:border-iron">
          <h2>{t('privy.maximum-privacy.title')}</h2>
          <p className="mt-2 text-iron dark:text-bombay">
            {t('privy.maximum-privacy.description')}
          </p>
          <Button
            onClick={() => {
              openUrl(NymVpnPricingUrl);
            }}
            className="mt-4"
          >
            <div className="flex items-center gap-2">
              {t('create-account')} <MsIcon icon="open_in_new" />
            </div>
          </Button>
        </div>

        <div className="py-6 flex flex-col gap-4">
          <h2>{t('privy.already-have-an-account.title')}</h2>
          <Button
            outline
            color="gray"
            onClick={() => {
              navigate(routes.passphraseLogin);
            }}
            className="group border border-iron dark:border-bombay hover:ring-0! dark:hover:ring-0!"
          >
            <span className="text-black dark:text-white group-hover:text-black/50 dark:group-hover:text-white/80">
              {t('privy.already-have-an-account.button')}
            </span>
          </Button>
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
                text={t('tos', { ns: 'common' })}
                url={ToSUrl}
                className="text-black dark:text-white"
                textClassName="underline-offset-2"
                data-testid="welcome-tos-link"
              />
            ),
            privacyLink: (
              <Link
                text={t('privacy-statement', { ns: 'common' })}
                url={PrivacyPolicyUrl}
                className="text-black dark:text-white"
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
