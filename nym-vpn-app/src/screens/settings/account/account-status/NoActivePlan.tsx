import { useTranslation } from 'react-i18next';
import { CardNewBody, MsIcon } from '../../../../ui';

export function NoActivePlan() {
  const { t } = useTranslation('account');
  return (
    <CardNewBody className="py-8">
      <div className="flex flex-col items-center justify-center gap-3 w-full">
        <div className="flex items-center justify-center w-14 h-14 rounded-full bg-faded-lavender dark:bg-mine-shaft border border-mercury dark:border-ash">
          <MsIcon
            icon="remove_moderator"
            className="text-text-secondary"
          />
        </div>
        <p className="text-base text-text-primary select-none">
          {t('account-status.no-plan')}
        </p>
      </div>
    </CardNewBody>
  );
}
