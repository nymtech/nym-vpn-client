import { DialogTitle } from '@headlessui/react';
import { TFunction } from 'i18next';
import { useTranslation } from 'react-i18next';
import { Button, Dialog, MsIcon } from '../../ui';
import { ExpiryReminder } from './expiryReminder';

export type Props = {
  isOpen: boolean;
  reminder: ExpiryReminder | null;
  onAction: () => void;
  onClose: () => void;
};

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

function ExpiryReminderDialog({ isOpen, reminder, onAction, onClose }: Props) {
  const { t } = useTranslation('notifications');
  if (!reminder) {
    return null;
  }
  const plan = reminder.isFreepass ? 'freepass' : 'subscription';

  return (
    <Dialog
      open={isOpen}
      onClose={onClose}
      className="flex flex-col items-center gap-6"
      data-testid="expiry-reminder-dialog"
    >
      <div className="flex flex-col items-center gap-4">
        <MsIcon
          icon="info"
          className="text-text-primary text-3xl"
          data-testid="expiry-reminder-dialog-info-icon"
        />
        <DialogTitle
          as="h3"
          className="text-text-primary text-xl"
          data-testid="expiry-reminder-dialog-title"
        >
          {reminderTitle(t, reminder)}
        </DialogTitle>
      </div>
      <p
        className="text-text-secondary"
        data-testid="expiry-reminder-dialog-description"
      >
        {t(`renewal-reminder.body.${plan}`)}
      </p>
      <Button
        variant="primary"
        onClick={onAction}
        className="mt-2"
        data-testid="expiry-reminder-dialog-button"
      >
        {t(
          reminder.isFreepass
            ? 'renewal-reminder.action.get-plan'
            : 'renewal-reminder.action.renew',
        )}
      </Button>
      <Button
        variant="outlined"
        onClick={onClose}
        data-testid="expiry-reminder-dialog-dismiss"
      >
        {t('renewal-reminder.action.not-now')}
      </Button>
    </Dialog>
  );
}

export default ExpiryReminderDialog;
