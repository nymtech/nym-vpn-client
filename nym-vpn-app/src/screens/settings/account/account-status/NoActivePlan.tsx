import { useTranslation } from 'react-i18next';
import { CardNewBody, MsIcon } from '../../../../ui';

export function NoActivePlan() {
  const { t } = useTranslation('account');
  return (
    <CardNewBody className="py-8">
      <div className="flex w-full flex-col items-center justify-center gap-3">
        <div className="bg-surface-bg dark:bg-surface-elev border-surface-elev dark:border-surface-bg flex h-14 w-14 items-center justify-center rounded-full border">
          <MsIcon icon="remove_moderator" className="text-text-secondary" />
        </div>
        <p className="text-text-primary text-base select-none">
          {t('account-status.no-plan')}
        </p>
      </div>
    </CardNewBody>
  );
}
