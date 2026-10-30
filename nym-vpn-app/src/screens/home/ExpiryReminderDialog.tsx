import { TFunction } from 'i18next';
import { useTranslation } from 'react-i18next';
import { ConfirmationDialog } from '../../ui';
import { ExpiryReminder } from './expiryReminder';
import useExpiryReminder from './useExpiryReminder';

function reminderTitle(
  t: TFunction<'notifications', undefined>,
  { tier, isFreepass, count }: ExpiryReminder,
): string {
  const plan = isFreepass ? 'freepass' : 'subscription';
  switch (tier) {
    case 'day7':
    case 'day3':
      return t(`renewal-reminder.days.${plan}`, { count });
    case 'hour24':
      return t(`renewal-reminder.hours.${plan}`, { count });
    default:
      return t(`renewal-reminder.expired.${plan}`);
  }
}

function ExpiryReminderDialog() {
  const { t } = useTranslation('notifications');
  const { reminder, isOpen, onAction, onClose } = useExpiryReminder();
  if (!reminder) {
    return null;
  }
  const plan = reminder.isFreepass ? 'freepass' : 'subscription';

  return (
    <ConfirmationDialog
      icon="info"
      title={reminderTitle(t, reminder)}
      description={t(`renewal-reminder.body.${plan}`)}
      confirmButtonText={t(
        reminder.isFreepass
          ? 'renewal-reminder.action.get-plan'
          : 'renewal-reminder.action.renew',
      )}
      cancelButtonText={t('renewal-reminder.action.not-now')}
      isOpen={isOpen}
      isLoading={false}
      onConfirm={onAction}
      onCancel={onClose}
    />
  );
}

export default ExpiryReminderDialog;
