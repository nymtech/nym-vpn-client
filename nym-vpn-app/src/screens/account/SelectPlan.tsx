import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Button, MsIcon, PageAnim } from '../../ui';
import { NymVpnPricingUrl } from '../../constants';
import { routes } from '../../router';

function SelectPlan() {
  const navigate = useNavigate();
  const { t } = useTranslation('account');

  const handleClick = () => {
    openUrl(NymVpnPricingUrl).finally(() => {
      navigate(routes.root);
    });
  };

  return (
    <PageAnim className="h-full flex flex-col items-center select-none cursor-default">
      <div className="grow flex flex-col justify-center items-center gap-6 px-4">
        <MsIcon className="text-malachite text-4xl" icon="verified_user" />
        <div className="flex flex-col gap-2 text-2xl text-center dark:text-white">
          <h1 className="truncate">{t('select-a-plan.title')}</h1>
        </div>
        <h3 className="text-center dark:text-bombay w-72">
          {t('select-a-plan.description-1')}
        </h3>
        <h3 className="text-center dark:text-bombay w-72">
          {t('select-a-plan.description-2')}
        </h3>
      </div>
      <Button className="w-full" onClick={handleClick}>
        {t('select-a-plan.button')}
      </Button>
    </PageAnim>
  );
}

export default SelectPlan;
