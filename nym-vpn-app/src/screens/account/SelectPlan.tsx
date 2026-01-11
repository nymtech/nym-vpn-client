import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Button, MsIcon, PageAnim } from '../../ui';
import { NymVpnAccountLoginUrl } from '../../constants';
import { routes } from '../../router';
import { CheckCircleIcon } from '../../assets';

function Feature({ icon, title }: { icon: string; title: string }) {
  return (
    <h3 className="text-left dark:text-bombay w-72 flex items-center gap-2 text-base">
      <MsIcon icon={icon} className="text-malachite" /> {title}
    </h3>
  );
}

function SelectPlan() {
  const navigate = useNavigate();
  const { t } = useTranslation('account');

  const handleClick = () => {
    openUrl(NymVpnAccountLoginUrl).finally(() => {
      navigate(routes.root);
    });
  };

  return (
    <PageAnim className="h-full flex flex-col items-center select-none cursor-default">
      <div className="grow flex flex-col justify-center items-center gap-6 px-4">
        <CheckCircleIcon className="text-malachite text-4xl" />
        <div className="flex flex-col gap-2 text-2xl text-center dark:text-white">
          <h1 className="truncate">{t('select-a-plan.title')}</h1>
        </div>
        <div className="flex flex-col gap-2 self-start">
          <Feature
            icon="verified_user"
            title={t('select-a-plan.features.all-included')}
          />
          <Feature icon="campaign" title={t('select-a-plan.features.no-ads')} />
          <Feature
            icon="chat_error"
            title={t('select-a-plan.features.cancel-anytime')}
          />
        </div>
      </div>
      <Button className="w-full" onClick={handleClick}>
        <span className="flex items-center gap-2">
          {t('select-a-plan.button')} <MsIcon icon="open_in_new" />
        </span>
      </Button>
    </PageAnim>
  );
}

export default SelectPlan;
