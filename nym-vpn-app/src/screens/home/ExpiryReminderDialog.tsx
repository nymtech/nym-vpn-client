import { DialogTitle } from '@headlessui/react';
import { Button, Dialog, MsIcon } from '../../ui';
import { ExpiryReminderDialogProps } from './useExpiryReminder';

function ExpiryReminderDialog({
  isOpen,
  title,
  body,
  actionLabel,
  dismissLabel,
  onAction,
  onClose,
}: ExpiryReminderDialogProps) {
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
          {title}
        </DialogTitle>
      </div>
      <p
        className="text-text-secondary"
        data-testid="expiry-reminder-dialog-description"
      >
        {body}
      </p>
      <Button
        variant="primary"
        onClick={onAction}
        className="mt-2"
        data-testid="expiry-reminder-dialog-button"
      >
        {actionLabel}
      </Button>
      <Button
        variant="outlined"
        onClick={onClose}
        data-testid="expiry-reminder-dialog-dismiss"
      >
        {dismissLabel}
      </Button>
    </Dialog>
  );
}

export default ExpiryReminderDialog;
